use crate::prelude::*;
use bevy::{ecs::system::SystemId, prelude::*};
use std::{cell::OnceCell, time::Instant};

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn egible_character(
    envir: Envir,
    mut timeouts: ResMut<Timeouts>,
    actors: Query<Actor>,
    players: Query<(), With<Player>>,
) -> Option<Entity> {
    let egible_entities = actors
        .iter()
        .filter(|a| envir.is_accessible(*a.pos) || players.get(a.entity).is_ok())
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    timeouts.next(&egible_entities)
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn plan_action(
    In(active_actor): In<Option<Entity>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut healing_writer: EventWriter<ActorEvent<Healing>>,
    player_action_state: Res<State<PlayerActionState>>,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    currently_visible_builder: CurrentlyVisibleBuilder,
    clock: Clock,
    mut instruction_queue: ResMut<InstructionQueue>,
    explored: Res<Explored>,
    actors: Query<Actor>,
    factions: Query<(&Pos, &Faction), With<Life>>,
    players: Query<(), With<Player>>,
) -> Option<(Entity, PlannedAction)> {
    let start = Instant::now();

    let Some(active_actor) = active_actor else {
        eprintln!("No egible characters!");
        return None;
    };

    let factions = &factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    let actor = actors
        .get(active_actor)
        .expect("'entity' should be a known actor");
    let enemies = actor
        .faction
        .enemies(&currently_visible_builder, factions, &actor);
    let action = if players.get(active_actor).is_ok() {
        player_action_state.plan_action(
            &mut next_player_action_state,
            &mut message_writer,
            &mut healing_writer,
            &currently_visible_builder.envir,
            &mut instruction_queue,
            &explored,
            &actor,
            clock.time(),
            &enemies,
        )?
    } else {
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
    };

    log_if_slow("plan_action", start);

    Some((actor.entity, action))
}

pub(in super::super) struct PerformSystems {
    stay: SystemId<ActionIn<Stay>, Impact>,
    step: SystemId<ActionIn<Step>, Impact>,
    attack: SystemId<ActionIn<Attack>, Impact>,
    smash: SystemId<ActionIn<Smash>, Impact>,
    pulp: SystemId<ActionIn<Pulp>, Impact>,
    close: SystemId<ActionIn<Close>, Impact>,
    wield: SystemId<ActionIn<ItemAction<Wield>>, Impact>,
    unwield: SystemId<ActionIn<ItemAction<Unwield>>, Impact>,
    pickup: SystemId<ActionIn<ItemAction<Pickup>>, Impact>,
    move_item: SystemId<ActionIn<ItemAction<MoveItem>>, Impact>,
    examine_item: SystemId<ActionIn<ItemAction<ExamineItem>>, ()>,
    change_pace: SystemId<ActionIn<ChangePace>, ()>,
}

impl PerformSystems {
    fn new(world: &mut World) -> Self {
        Self {
            stay: world.register_system(perform_stay),
            step: world.register_system(perform_step),
            attack: world.register_system(perform_attack),
            smash: world.register_system(perform_smash),
            pulp: world.register_system(perform_pulp),
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

    let impact = match planned_action {
        PlannedAction::Stay { duration } => perform_action_inner(
            perform_systems.stay,
            world,
            ActionIn::new(actor_entity, Stay { duration }),
        ),
        PlannedAction::Step { to } => perform_action_inner(
            perform_systems.step,
            world,
            ActionIn::new(actor_entity, Step { to }),
        ),
        PlannedAction::Attack { target } => perform_action_inner(
            perform_systems.attack,
            world,
            ActionIn::new(actor_entity, Attack { target }),
        ),
        PlannedAction::Smash { target } => perform_action_inner(
            perform_systems.smash,
            world,
            ActionIn::new(actor_entity, Smash { target }),
        ),
        PlannedAction::Pulp { target } => perform_action_inner(
            perform_systems.pulp,
            world,
            ActionIn::new(actor_entity, Pulp { target }),
        ),
        PlannedAction::Close { target } => perform_action_inner(
            perform_systems.close,
            world,
            ActionIn::new(actor_entity, Close { target }),
        ),
        PlannedAction::Wield { item } => perform_action_inner(
            perform_systems.wield,
            world,
            ActionIn::new(actor_entity, ItemAction::new(item, Wield)),
        ),
        PlannedAction::Unwield { item } => perform_action_inner(
            perform_systems.unwield,
            world,
            ActionIn::new(actor_entity, ItemAction::new(item, Unwield)),
        ),
        PlannedAction::Pickup { item } => perform_action_inner(
            perform_systems.pickup,
            world,
            ActionIn::new(actor_entity, ItemAction::new(item, Pickup)),
        ),
        PlannedAction::MoveItem { item, to } => perform_action_inner(
            perform_systems.move_item,
            world,
            ActionIn::new(actor_entity, ItemAction::new(item, MoveItem { to })),
        ),
        PlannedAction::ExamineItem { item } => {
            perform_action_inner(
                perform_systems.examine_item,
                world,
                ActionIn::new(actor_entity, ItemAction::new(item, ExamineItem)),
            );
            return None;
        }
        PlannedAction::ChangePace => {
            perform_action_inner(
                perform_systems.change_pace,
                world,
                ActionIn::new(actor_entity, ChangePace),
            );
            return None;
        }
    };

    log_if_slow("perform_action", start);

    Some(impact)
}

fn perform_action_inner<A, R>(
    system_id: SystemId<ActionIn<A>, R>,
    world: &mut World,
    action_in: ActionIn<A>,
) -> R
where
    A: Action,
    R: 'static,
{
    world
        .run_system_with_input(system_id, action_in)
        .expect("Action should have succeeded")
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub(in super::super) fn perform_stay(In(stay): In<ActionIn<Stay>>, actors: Query<Actor>) -> Impact {
    stay.actor(&actors).stay(&stay.action)
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_step(
    In(step): In<ActionIn<Step>>,
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    mut envir: Envir,
    actors: Query<Actor>,
) -> Impact {
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
) -> Impact {
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
) -> Impact {
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
) -> Impact {
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
pub(in super::super) fn perform_close(
    In(close): In<ActionIn<Close>>,
    mut message_writer: MessageWriter,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    envir: Envir,
    actors: Query<Actor>,
) -> Impact {
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
) -> Impact {
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
) -> Impact {
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
) -> Impact {
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
) -> Impact {
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
) {
    ActorItem::examine_item(
        &mut message_writer,
        &infos,
        &examine_item.action.item(&items),
    );
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn perform_change_pace(
    In(change_pace): In<ActionIn<ChangePace>>,
    mut commands: Commands,
    actors: Query<Actor>,
) {
    change_pace.actor(&actors).change_pace(&mut commands);
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
