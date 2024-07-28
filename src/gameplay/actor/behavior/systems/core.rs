use crate::prelude::*;
use bevy::{ecs::system::SystemId, prelude::*};
use std::{cell::OnceCell, time::Instant};

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn egible_character(
    envir: Envir,
    mut timeouts: ResMut<Timeouts>,
    actors: Query<Actor>,
    players: Query<(), With<Player>>,
) -> Entity {
    let egible_entities = actors
        .iter()
        .filter(|a| envir.is_accessible(*a.pos) || players.get(a.entity).is_ok())
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    timeouts
        .next(&egible_entities)
        .expect("There should be an egible character")
}

pub(in super::super) struct PlanSystems {
    manual_player_action: SystemId<Entity, Option<PlannedAction>>,
    automatic_player_action: SystemId<Entity, Option<PlannedAction>>,
    wait_for_player_input: SystemId<(), ()>,
    npc_action: SystemId<Entity, PlannedAction>,
}

impl PlanSystems {
    fn new(world: &mut World) -> Self {
        Self {
            manual_player_action: world.register_system(plan_manual_player_action),
            automatic_player_action: world.register_system(plan_automatic_player_action),
            wait_for_player_input: world.register_system(wait_for_player_input),
            npc_action: world.register_system(plan_npc_action),
        }
    }
}
#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn plan_action(
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

#[allow(clippy::needless_pass_by_value)]
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
    let actor = actors
        .get(active_actor)
        .expect("'entity' should be a known actor");
    player_action_state.plan_manual_action(
        &mut next_player_action_state,
        &mut message_writer,
        &currently_visible_builder.envir,
        &mut instruction_queue,
        &actor,
        clock.time(),
    )
}

#[allow(clippy::needless_pass_by_value)]
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
    let actor = actors
        .get(active_actor)
        .expect("'entity' should be a known actor");

    let factions = &factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    player_action_state.plan_automatic_action(
        &currently_visible_builder,
        &mut instruction_queue,
        &explored,
        &actor,
        clock.time(),
        factions,
    )
}

#[allow(clippy::needless_pass_by_value)]
fn wait_for_player_input(mut instruction_queue: ResMut<InstructionQueue>) {
    instruction_queue.start_waiting();
    println!("Waiting for user action");
}

#[allow(clippy::needless_pass_by_value)]
fn plan_npc_action(
    In(active_actor): In<Entity>,
    mut commands: Commands,
    currently_visible_builder: CurrentlyVisibleBuilder,
    actors: Query<Actor>,
    factions: Query<(&Pos, &Faction), With<Life>>,
) -> PlannedAction {
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
    println!(
        "{} at {:?} plans {:?} and does {:?}",
        actor.name.single(*actor.pos).text,
        actor.pos,
        strategy.intent,
        strategy.action
    );

    strategy.action
}

pub(in super::super) struct PerformSystems {
    stay: SystemId<ActionIn<Stay>, Option<Impact>>,
    sleep: SystemId<ActionIn<Sleep>, Option<Impact>>,
    step: SystemId<ActionIn<Step>, Option<Impact>>,
    attack: SystemId<ActionIn<Attack>, Option<Impact>>,
    smash: SystemId<ActionIn<Smash>, Option<Impact>>,
    pulp: SystemId<ActionIn<Pulp>, Option<Impact>>,
    peek: SystemId<ActionIn<Peek>, Option<Impact>>,
    close: SystemId<ActionIn<Close>, Option<Impact>>,
    wield: SystemId<ActionIn<ItemAction<Wield>>, Option<Impact>>,
    unwield: SystemId<ActionIn<ItemAction<Unwield>>, Option<Impact>>,
    pickup: SystemId<ActionIn<ItemAction<Pickup>>, Option<Impact>>,
    move_item: SystemId<ActionIn<ItemAction<MoveItem>>, Option<Impact>>,
    examine_item: SystemId<ActionIn<ItemAction<ExamineItem>>, Option<Impact>>,
    change_pace: SystemId<ActionIn<ChangePace>, Option<Impact>>,
}

impl PerformSystems {
    fn new(world: &mut World) -> Self {
        Self {
            stay: world.register_system(perform_stay),
            sleep: world.register_system(perform_sleep),
            step: world.register_system(perform_step),
            attack: world.register_system(perform_attack),
            smash: world.register_system(perform_smash),
            pulp: world.register_system(perform_pulp),
            peek: world.register_system(perform_peek),
            close: world.register_system(perform_close),
            wield: world.register_system(perform_wield),
            unwield: world.register_system(perform_unwield),
            pickup: world.register_system(perform_pickup),
            move_item: world.register_system(perform_move_item),
            examine_item: world.register_system(perform_examine_item),
            change_pace: world.register_system(perform_change_pace),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_action(
    In(option): In<Option<(Entity, PlannedAction)>>,
    world: &mut World,
    perform_systems: Local<OnceCell<PerformSystems>>,
) -> Option<Impact> {
    let start = Instant::now();

    let Some((actor_entity, planned_action)) = option else {
        // No egible characters
        return None;
    };

    let perform_systems = perform_systems.get_or_init(|| PerformSystems::new(world));

    let perform_fn = match planned_action {
        PlannedAction::Stay => act_fn(perform_systems.stay, Stay),
        PlannedAction::Sleep => act_fn(perform_systems.sleep, Sleep),
        PlannedAction::Step { to } => act_fn(perform_systems.step, Step { to }),
        PlannedAction::Attack { target } => act_fn(perform_systems.attack, Attack { target }),
        PlannedAction::Smash { target } => act_fn(perform_systems.smash, Smash { target }),
        PlannedAction::Pulp { target } => act_fn(perform_systems.pulp, Pulp { target }),
        PlannedAction::Peek { target } => act_fn(perform_systems.peek, Peek { target }),
        PlannedAction::Close { target } => act_fn(perform_systems.close, Close { target }),
        PlannedAction::Wield { item } => {
            act_fn(perform_systems.wield, ItemAction::new(item, Wield))
        }
        PlannedAction::Unwield { item } => {
            act_fn(perform_systems.unwield, ItemAction::new(item, Unwield))
        }
        PlannedAction::Pickup { item } => {
            act_fn(perform_systems.pickup, ItemAction::new(item, Pickup))
        }
        PlannedAction::MoveItem { item, to } => act_fn(
            perform_systems.move_item,
            ItemAction::new(item, MoveItem { to }),
        ),
        PlannedAction::ExamineItem { item } => act_fn(
            perform_systems.examine_item,
            ItemAction::new(item, ExamineItem),
        ),
        PlannedAction::ChangePace => act_fn(perform_systems.change_pace, ChangePace),
    };

    let impact = perform_fn(world, actor_entity);

    log_if_slow("perform_action", start);

    impact
}

fn act_fn<A: Action>(
    system: SystemId<ActionIn<A>, Option<Impact>>,
    action: A,
) -> Box<dyn FnOnce(&mut World, Entity) -> Option<Impact>> {
    Box::new(move |world, actor_entity| {
        run_system(world, system, ActionIn::new(actor_entity, action))
    })
}

fn run_system<I: 'static, R: 'static>(world: &mut World, system_id: SystemId<I, R>, in_: I) -> R {
    world
        .run_system_with_input(system_id, in_)
        .expect("Action should have succeeded")
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub(in super::super) fn perform_stay(
    In(stay): In<ActionIn<Stay>>,
    actors: Query<Actor>,
) -> Option<Impact> {
    Some(stay.actor(&actors).stay())
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub(in super::super) fn perform_sleep(
    In(sleep): In<ActionIn<Sleep>>,
    mut healing_writer: EventWriter<ActorEvent<Healing>>,
    mut healing_durations: Query<&mut HealingDuration>,
    actors: Query<Actor>,
) -> Option<Impact> {
    Some(
        sleep
            .actor(&actors)
            .sleep(&mut healing_writer, &mut healing_durations),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_step(
    In(step): In<ActionIn<Step>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    mut envir: Envir,
    actors: Query<Actor>,
) -> Option<Impact> {
    step.actor(&actors).step(
        &mut commands,
        &mut message_writer,
        &mut toggle_writer,
        &mut envir,
        &step.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_attack(
    In(attack): In<ActionIn<Attack>>,
    mut message_writer: MessageWriter,
    mut damage_writer: EventWriter<ActorEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    attack.actor(&actors).attack(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &attack.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_smash(
    In(smash): In<ActionIn<Smash>>,
    mut message_writer: MessageWriter,
    mut damage_writer: EventWriter<TerrainEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    smash.actor(&actors).smash(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &smash.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_pulp(
    In(pulp): In<ActionIn<Pulp>>,
    mut message_writer: MessageWriter,
    mut corpse_damage_writer: EventWriter<CorpseEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    pulp.actor(&actors).pulp(
        &mut message_writer,
        &mut corpse_damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &pulp.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_peek(
    In(peek): In<ActionIn<Peek>>,
    mut message_writer: MessageWriter,
    mut player_action_state: ResMut<NextState<PlayerActionState>>,
    envir: Envir,
    actors: Query<Actor>,
) -> Option<Impact> {
    peek.actor(&actors).peek(
        &mut message_writer,
        &mut player_action_state,
        &envir,
        &peek.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_close(
    In(close): In<ActionIn<Close>>,
    mut message_writer: MessageWriter,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    envir: Envir,
    actors: Query<Actor>,
) -> Option<Impact> {
    close.actor(&actors).close(
        &mut message_writer,
        &mut toggle_writer,
        &envir,
        &close.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_wield(
    In(wield): In<ActionIn<ItemAction<Wield>>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Option<Impact> {
    wield.actor(&actors).wield(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &wield.action.item(&items),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_unwield(
    In(unwield): In<ActionIn<ItemAction<Unwield>>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Option<Impact> {
    unwield.actor(&actors).unwield(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &unwield.action.item(&items),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_pickup(
    In(pickup): In<ActionIn<ItemAction<Pickup>>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Option<Impact> {
    pickup.actor(&actors).pickup(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &pickup.action.item(&items),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_move_item(
    In(move_item): In<ActionIn<ItemAction<MoveItem>>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut location: ResMut<Location>,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Option<Impact> {
    move_item.actor(&actors).move_item(
        &mut commands,
        &mut message_writer,
        &subzone_level_entities,
        &mut location,
        &move_item.action.item(&items),
        move_item.action.change.to,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_examine_item(
    In(examine_item): In<ActionIn<ItemAction<ExamineItem>>>,
    mut message_writer: MessageWriter,
    infos: Res<Infos>,
    items: Query<Item>,
) -> Option<Impact> {
    ActorItem::examine_item(
        &mut message_writer,
        &infos,
        &examine_item.action.item(&items),
    );
    None
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_change_pace(
    In(change_pace): In<ActionIn<ChangePace>>,
    mut commands: Commands,
    actors: Query<Actor>,
) -> Option<Impact> {
    change_pace.actor(&actors).change_pace(&mut commands);
    None
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn proces_impact(
    In(impact): In<Option<Impact>>,
    mut message_writer: MessageWriter,
    mut stamina_impact_events: EventWriter<ActorEvent<StaminaImpact>>,
    mut timeouts: ResMut<Timeouts>,
    players: Query<Entity, With<Player>>,
) {
    let start = Instant::now();

    let Some(impact) = impact else {
        return;
    };

    impact.check_validity();
    let Some(stamina_impact) = impact.stamina_impact else {
        if players.get(impact.actor_entity).is_err() {
            message_writer.str("NPC action failed").send_error();
            // To prevent the application hanging on failing NPC actions, we add a small timeout
            timeouts.add(impact.actor_entity, Milliseconds(1000));
        }

        return;
    };

    timeouts.add(impact.actor_entity, impact.timeout);

    stamina_impact_events.send(ActorEvent::new(impact.actor_entity, stamina_impact));

    log_if_slow("proces_impact", start);
}
