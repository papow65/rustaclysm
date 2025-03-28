use crate::gameplay::{spawn::TileSpawner, *};
use bevy::ecs::schedule::{IntoScheduleConfigs as _, ScheduleConfigs};
use bevy::ecs::system::{ScheduleSystem, SystemId};
use bevy::prelude::{
    Commands, Entity, EventWriter, In, IntoSystem as _, Local, NextState, Query, Res, ResMut,
    Single, State, StateTransition, SystemInput, With, World, debug,
};
use std::{cell::OnceCell, time::Instant};
use units::Duration;
use util::log_if_slow;

pub(in super::super) fn perform_egible_character_action() -> ScheduleConfigs<ScheduleSystem> {
    egible_character
        .pipe(plan_action)
        .pipe(perform_action)
        .pipe(proces_impact)
        .into_configs()
}

#[expect(clippy::needless_pass_by_value)]
fn egible_character(
    envir: Envir,
    mut timeouts: ResMut<Timeouts>,
    actors: Query<Actor>,
    player: Single<Entity, With<Player>>,
) -> Entity {
    let egible_entities = actors
        .iter()
        .filter(|a| envir.is_accessible(*a.pos) || *player == a.entity)
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    timeouts
        .next(&egible_entities)
        .expect("There should be an egible character")
}

struct PlanSystems {
    manual_player_action: SystemId<In<Entity>, Option<PlannedAction>>,
    automatic_player_action: SystemId<In<Entity>, Option<PlannedAction>>,
    wait_for_player_input: SystemId<(), ()>,
    npc_action: SystemId<In<Entity>, PlannedAction>,
}

impl PlanSystems {
    fn new(world: &mut World) -> Self {
        Self {
            manual_player_action: world.register_system_cached(plan_manual_player_action),
            automatic_player_action: world.register_system_cached(plan_automatic_player_action),
            wait_for_player_input: world.register_system_cached(wait_for_player_input),
            npc_action: world.register_system_cached(plan_npc_action),
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn plan_action(
    In(active_actor): In<Entity>,
    world: &mut World,
    plan_systems: Local<OnceCell<PlanSystems>>,
) -> Option<(Entity, PlannedAction)> {
    let start = Instant::now();

    let plan_systems = plan_systems.get_or_init(|| PlanSystems::new(world));

    let player_active = world
        .get_entity(active_actor)
        .expect("The active actor should exist")
        .contains::<Player>();

    let action = if player_active {
        let manual_action = run_system(world, plan_systems.manual_player_action, active_actor);
        // The `PlannedActionState` could have transitioned with or without a 'manual_action'.
        world.run_schedule(StateTransition);
        manual_action
            .or_else(|| run_system(world, plan_systems.automatic_player_action, active_actor))
            .or_else(|| {
                run_system(world, plan_systems.wait_for_player_input, ());
                None
            })?
    } else {
        run_system(world, plan_systems.npc_action, active_actor)
    };

    log_if_slow("plan_action", start);

    Some((active_actor, action))
}

#[expect(clippy::needless_pass_by_value)]
fn plan_manual_player_action(
    In(active_actor): In<Entity>,
    mut message_writer: MessageWriter,
    player_action_state: Res<State<PlayerActionState>>,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    currently_visible_builder: CurrentlyVisibleBuilder,
    clock: Clock,
    mut instruction_queue: ResMut<InstructionQueue>,
    actors: Query<Actor>,
) -> Option<PlannedAction> {
    let start = Instant::now();

    let actor = actors
        .get(active_actor)
        .expect("'entity' should be a known actor");
    let impact = player_action_state.plan_manual_action(
        &mut next_player_action_state,
        &mut message_writer,
        &currently_visible_builder.envir,
        &mut instruction_queue,
        &actor,
        clock.time(),
    );

    log_if_slow("plan_manual_player_action", start);

    impact
}

#[expect(clippy::needless_pass_by_value)]
fn plan_automatic_player_action(
    In(active_actor): In<Entity>,
    player_action_state: Res<State<PlayerActionState>>,
    currently_visible_builder: CurrentlyVisibleBuilder,
    clock: Clock,
    mut instruction_queue: ResMut<InstructionQueue>,
    explored: Res<Explored>,
    actors: Query<Actor>,
    factions: Query<(&Pos, &Faction), With<Life>>,
) -> Option<PlannedAction> {
    let start = Instant::now();

    let actor = actors
        .get(active_actor)
        .expect("'entity' should be a known actor");

    let factions = &factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    let planned_action = player_action_state.plan_automatic_action(
        &currently_visible_builder,
        &mut instruction_queue,
        &explored,
        &actor,
        clock.time(),
        factions,
    );

    log_if_slow("plan_manual_player_action", start);

    planned_action
}

fn wait_for_player_input(mut instruction_queue: ResMut<InstructionQueue>) {
    let start = Instant::now();

    instruction_queue.start_waiting();

    log_if_slow("wait_for_player_input", start);
}

#[expect(clippy::needless_pass_by_value)]
fn plan_npc_action(
    In(active_actor): In<Entity>,
    mut commands: Commands,
    currently_visible_builder: CurrentlyVisibleBuilder,
    actors: Query<Actor>,
    factions: Query<(&Pos, &Faction), With<Life>>,
) -> PlannedAction {
    let start = Instant::now();

    let actor = actors
        .get(active_actor)
        .expect("'entity' should be a known actor");
    let factions = &factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    let enemies = actor
        .faction
        .enemies(&currently_visible_builder, factions, &actor);
    let strategy =
        actor
            .faction
            .strategize(&currently_visible_builder.envir, factions, &enemies, &actor);
    if let Some(last_enemy) = strategy.last_enemy {
        commands.entity(actor.entity).insert(last_enemy);
    }
    debug!(
        "{} at {:?} plans {:?} and does {:?}",
        actor.name.single(*actor.pos).text,
        actor.pos,
        strategy.intent,
        strategy.action
    );

    log_if_slow("plan_npc_action", start);

    strategy.action
}

struct PerformSystems {
    stay: SystemId<In<ActionIn<Stay>>, ActorImpact>,
    sleep: SystemId<In<ActionIn<Sleep>>, ActorImpact>,
    step: SystemId<In<ActionIn<Step>>, ActorImpact>,
    attack: SystemId<In<ActionIn<Attack>>, ActorImpact>,
    smash: SystemId<In<ActionIn<Smash>>, ActorImpact>,
    pulp: SystemId<In<ActionIn<Pulp>>, ActorImpact>,
    peek: SystemId<In<ActionIn<Peek>>, ActorImpact>,
    close: SystemId<In<ActionIn<Close>>, ActorImpact>,
    wield: SystemId<In<ActionIn<Wield>>, ActorImpact>,
    unwield: SystemId<In<ActionIn<Unwield>>, ActorImpact>,
    pickup: SystemId<In<ActionIn<Pickup>>, ActorImpact>,
    move_item: SystemId<In<ActionIn<MoveItem>>, ActorImpact>,
    start_craft: SystemId<In<ActionIn<StartCraft>>, ActorImpact>,
    continue_craft: SystemId<In<ActionIn<ContinueCraft>>, ActorImpact>,
    examine_item: SystemId<In<ActionIn<ExamineItem>>, ActorImpact>,
    change_pace: SystemId<In<ActionIn<ChangePace>>, ActorImpact>,
}

impl PerformSystems {
    fn new(world: &mut World) -> Self {
        Self {
            stay: world.register_system_cached(perform_stay),
            sleep: world.register_system_cached(perform_sleep),
            step: world.register_system_cached(perform_step),
            attack: world.register_system_cached(perform_attack),
            smash: world.register_system_cached(perform_smash),
            pulp: world.register_system_cached(perform_pulp),
            peek: world.register_system_cached(perform_peek),
            close: world.register_system_cached(perform_close),
            wield: world.register_system_cached(perform_wield),
            unwield: world.register_system_cached(perform_unwield),
            pickup: world.register_system_cached(perform_pickup),
            move_item: world.register_system_cached(perform_move_item),
            start_craft: world.register_system_cached(perform_start_craft),
            continue_craft: world.register_system_cached(perform_continue_craft),
            examine_item: world.register_system_cached(perform_examine_item),
            change_pace: world.register_system_cached(perform_change_pace),
        }
    }

    fn perform(
        &self,
        world: &mut World,
        planned_action: PlannedAction,
        actor_entity: Entity,
    ) -> ActorImpact {
        let perform_fn = match planned_action {
            PlannedAction::Stay => act_fn(self.stay, Stay),
            PlannedAction::Sleep => act_fn(self.sleep, Sleep),
            PlannedAction::Step(step) => act_fn(self.step, step),
            PlannedAction::Attack(attack) => act_fn(self.attack, attack),
            PlannedAction::Smash(smash) => act_fn(self.smash, smash),
            PlannedAction::Pulp(pulp) => act_fn(self.pulp, pulp),
            PlannedAction::Peek(peek) => act_fn(self.peek, peek),
            PlannedAction::Close(close) => act_fn(self.close, close),
            PlannedAction::Wield(wield) => act_fn(self.wield, wield),
            PlannedAction::Unwield(unwield) => act_fn(self.unwield, unwield),
            PlannedAction::Pickup(pickup) => act_fn(self.pickup, pickup),
            PlannedAction::MoveItem(move_item) => act_fn(self.move_item, move_item),
            PlannedAction::StartCraft(start_craft) => act_fn(self.start_craft, start_craft),
            PlannedAction::ContinueCraft(continue_craft) => {
                act_fn(self.continue_craft, continue_craft)
            }
            PlannedAction::ExamineItem(examine_item) => act_fn(self.examine_item, examine_item),
            PlannedAction::ChangePace(change_pace) => act_fn(self.change_pace, change_pace),
        };

        perform_fn(world, actor_entity)
    }
}

fn act_fn<A: Action>(
    system: SystemId<In<ActionIn<A>>, ActorImpact>,
    action: A,
) -> Box<dyn FnOnce(&mut World, Entity) -> ActorImpact> {
    Box::new(move |world, actor_entity| {
        run_system(world, system, ActionIn::new(actor_entity, action))
    })
}

#[expect(clippy::needless_pass_by_value)]
fn perform_action(
    In(option): In<Option<(Entity, PlannedAction)>>,
    world: &mut World,
    perform_systems: Local<OnceCell<PerformSystems>>,
) -> Option<ActorImpact> {
    let start = Instant::now();

    let Some((actor_entity, planned_action)) = option else {
        // No egible characters
        return None;
    };

    let impact = perform_systems
        .get_or_init(|| PerformSystems::new(world))
        .perform(world, planned_action, actor_entity);

    log_if_slow("perform_action", start);

    Some(impact)
}

fn run_system<I: SystemInput + 'static, R: 'static>(
    world: &mut World,
    system_id: SystemId<I, R>,
    in_: I::Inner<'_>,
) -> R {
    world
        .run_system_with(system_id, in_)
        .expect("Action should have succeeded")
}

fn perform_stay(In(stay): In<ActionIn<Stay>>, actors: Query<Actor>) -> ActorImpact {
    stay.actor(&actors).stay()
}

#[expect(clippy::needless_pass_by_value)]
fn perform_sleep(
    In(sleep): In<ActionIn<Sleep>>,
    mut message_writer: MessageWriter,
    mut healing_writer: EventWriter<ActorEvent<Healing>>,
    player_action_state: Res<State<PlayerActionState>>,
    clock: Clock,
    mut healing_durations: Query<&mut HealingDuration>,
    actors: Query<Actor>,
) -> ActorImpact {
    sleep.actor(&actors).sleep(
        &mut message_writer,
        &mut healing_writer,
        player_action_state.get(),
        &clock,
        &mut healing_durations,
    )
}

fn perform_step(
    In(step): In<ActionIn<Step>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    mut envir: Envir,
    actors: Query<Actor>,
) -> ActorImpact {
    step.actor(&actors).step(
        &mut commands,
        &mut message_writer,
        &mut toggle_writer,
        &mut envir,
        &step.action,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_attack(
    In(attack): In<ActionIn<Attack>>,
    mut message_writer: MessageWriter,
    mut damage_writer: EventWriter<ActorEvent<Damage>>,
    envir: Envir,
    hierarchy: ItemHierarchy,
    actors: Query<Actor>,
) -> ActorImpact {
    attack.actor(&actors).attack(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &hierarchy,
        &attack.action,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_smash(
    In(smash): In<ActionIn<Smash>>,
    mut message_writer: MessageWriter,
    mut damage_writer: EventWriter<TerrainEvent<Damage>>,
    envir: Envir,
    hierarchy: ItemHierarchy,
    actors: Query<Actor>,
) -> ActorImpact {
    smash.actor(&actors).smash(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &hierarchy,
        &smash.action,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_pulp(
    In(pulp): In<ActionIn<Pulp>>,
    mut message_writer: MessageWriter,
    mut corpse_damage_writer: EventWriter<CorpseEvent<Damage>>,
    envir: Envir,
    hierarchy: ItemHierarchy,
    actors: Query<Actor>,
) -> ActorImpact {
    pulp.actor(&actors).pulp(
        &mut message_writer,
        &mut corpse_damage_writer,
        &envir,
        &hierarchy,
        &pulp.action,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_peek(
    In(peek): In<ActionIn<Peek>>,
    mut message_writer: MessageWriter,
    mut player_action_state: ResMut<NextState<PlayerActionState>>,
    envir: Envir,
    actors: Query<Actor>,
) -> ActorImpact {
    peek.actor(&actors).peek(
        &mut message_writer,
        &mut player_action_state,
        &envir,
        &peek.action,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_close(
    In(close): In<ActionIn<Close>>,
    mut message_writer: MessageWriter,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    envir: Envir,
    actors: Query<Actor>,
) -> ActorImpact {
    close.actor(&actors).close(
        &mut message_writer,
        &mut toggle_writer,
        &envir,
        &close.action,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_wield(
    In(wield): In<ActionIn<Wield>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    hierarchy: ItemHierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> ActorImpact {
    wield.actor(&actors).wield(
        &mut commands,
        &mut message_writer,
        &hierarchy,
        &wield.action.item(&items),
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_unwield(
    In(unwield): In<ActionIn<Unwield>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    hierarchy: ItemHierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> ActorImpact {
    unwield.actor(&actors).unwield(
        &mut commands,
        &mut message_writer,
        &hierarchy,
        &unwield.action.item(&items),
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_pickup(
    In(pickup): In<ActionIn<Pickup>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    hierarchy: ItemHierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> ActorImpact {
    pickup.actor(&actors).pickup(
        &mut commands,
        &mut message_writer,
        &hierarchy,
        &pickup.action.item(&items),
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_move_item(
    In(move_item): In<ActionIn<MoveItem>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut location: ResMut<Location>,
    actors: Query<Actor>,
    items: Query<Item>,
) -> ActorImpact {
    move_item.actor(&actors).move_item(
        &mut commands,
        &mut message_writer,
        &subzone_level_entities,
        &mut location,
        &move_item.action.item(&items),
        move_item.action.to,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn perform_start_craft(
    In(start_craft): In<ActionIn<StartCraft>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    mut spawner: TileSpawner,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    actors: Query<Actor>,
    mut amounts: Query<&mut Amount>,
) -> ActorImpact {
    start_craft.actor(&actors).start_craft(
        &mut commands,
        &mut message_writer,
        &mut next_player_action_state,
        &mut spawner,
        &subzone_level_entities,
        &mut amounts,
        &start_craft.action,
    )
}

fn perform_continue_craft(
    In(continue_craft): In<ActionIn<ContinueCraft>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    mut spawner: TileSpawner,
    actors: Query<Actor>,
    mut items: Query<(Item, &mut Craft)>,
) -> ActorImpact {
    continue_craft.actor(&actors).continue_craft(
        &mut commands,
        &mut message_writer,
        &mut next_player_action_state,
        &mut spawner,
        &mut items,
        continue_craft.action.item_entity,
    )
}

fn perform_examine_item(
    In(examine_item): In<ActionIn<ExamineItem>>,
    mut message_writer: MessageWriter,
    actors: Query<Actor>,
    items: Query<Item>,
) -> ActorImpact {
    examine_item
        .actor(&actors)
        .examine_item(&mut message_writer, &examine_item.action.item(&items))
}

fn perform_change_pace(
    In(change_pace): In<ActionIn<ChangePace>>,
    mut commands: Commands,
    actors: Query<Actor>,
) -> ActorImpact {
    change_pace
        .actor(&actors)
        .change_pace(&mut commands, change_pace.action)
}

#[expect(clippy::needless_pass_by_value)]
fn proces_impact(
    In(actor_impact): In<Option<ActorImpact>>,
    mut message_writer: MessageWriter,
    mut timeouts: ResMut<Timeouts>,
    mut staminas: Query<&mut Stamina>,
    player: Single<Entity, With<Player>>,
) {
    let start = Instant::now();

    let Some(actor_impact) = actor_impact else {
        // No egible characters, usually because we are waiting for user input
        return;
    };

    let actor = actor_impact.actor_entity;
    if let Some(impact) = actor_impact.impact {
        timeouts.add(actor, impact.duration());
        if let Ok(mut stamina) = staminas.get_mut(actor) {
            stamina.apply(&impact);
        }
    } else if *player != actor {
        message_writer.str("NPC action failed").send_error();
        // To prevent the application hanging on failing NPC actions, we add a small timeout
        timeouts.add(actor, Duration::SECOND);
    }

    log_if_slow("proces_impact", start);
}
