use super::log_if_slow;
use crate::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*};
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_game_over(
    mut app_exit_events: ResMut<bevy::ecs::event::Events<bevy::app::AppExit>>,
    dead_players: Query<(), (With<Player>, Without<Health>)>,
) {
    let start = Instant::now();

    if dead_players.get_single().is_ok() {
        app_exit_events.send(bevy::app::AppExit);
    }

    log_if_slow("manage_game_over", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn plan_action(
    mut commands: Commands,
    mut envir: Envir,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut timeouts: ResMut<Timeouts>,
    actors: Actors,
    mut players: Query<&mut Player>,
) -> Option<(Entity, Action)> {
    let start = Instant::now();

    let egible_entities = actors
        .actors()
        .filter(|a| envir.is_accessible(a.pos) || players.get(a.entity).is_ok())
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    let Some(active_entity) = timeouts.next(&egible_entities) else {
        eprintln!("No egible characters!");
        return None;
    };
    let factions = actors.collect_factions();
    let actor = actors.get(active_entity);
    let action = if let Ok(ref mut player) = players.get_mut(active_entity) {
        if let Some(action) = player.plan_action(
            &mut commands,
            &mut envir,
            &mut instruction_queue,
            actor.pos,
            timeouts.time(),
        ) {
            action
        } else if let PlayerActionState::Waiting(until) = player.state {
            if until <= timeouts.time() {
                instruction_queue.add(QueuedInstruction::Cancel);
                return None; // process the cancellation next turn
            } else {
                Action::Stay
            }
        } else {
            instruction_queue.start_waiting();
            println!("Waiting for user action");
            return None; // no key pressed - wait for the user
        }
    } else {
        let strategy = actor.faction.strategize(&envir, &factions, &actor);
        if let Some(last_enemy) = strategy.last_enemy {
            commands.entity(actor.entity).insert(last_enemy);
        }
        println!(
            "{} at {:?} plans {:?} and does {:?}",
            actor.label, actor.pos, strategy.intent, strategy.action
        );
        strategy.action
    };

    log_if_slow("plan_action", start);

    Some((actor.entity, action))
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_action(
    In(option): In<Option<(Entity, Action)>>,
    world: &mut World,
) -> Option<(Entity, Milliseconds)> {
    let start = Instant::now();

    let Some((actor_entity, action)) = option  else {
        return None;
    };

    let timeout = match action {
        Action::Stay => perform(world, actor_entity, |actor| actor.stay()),
        Action::Step { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.move_(commands, envir, target)
            })
        }
        Action::Attack { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.attack(commands, envir, target)
            })
        }
        Action::Smash { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.smash(commands, envir, target)
            })
        }
        Action::Close { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.close(commands, envir, target)
            })
        }
        Action::Wield => perform_commands_location_hierarchy(
            world,
            actor_entity,
            |actor, commands, location, hierarchy| actor.wield(commands, location, hierarchy),
        ),
        Action::Pickup => perform_commands_location_hierarchy(
            world,
            actor_entity,
            |actor, commands, location, hierarchy| actor.pickup(commands, location, hierarchy),
        ),
        Action::Dump => {
            let mut system_state = SystemState::<(
                Commands,
                Envir,
                Query<(Entity, &TextLabel, &Parent)>,
                Actors,
            )>::new(world);
            let (mut commands, mut envir, dumpees, actors) = system_state.get_mut(world);
            let duration =
                actors
                    .get(actor_entity)
                    .dump(&mut commands, &mut envir.location, &dumpees);
            system_state.apply(world);
            duration
        }
        Action::SwitchRunning => {
            perform_commands_envir(world, actor_entity, |actor, commands, _| {
                actor.switch_running(commands)
            })
        }
    };

    log_if_slow("manage_characters", start);

    Some((actor_entity, timeout))
}

// TODO combine all versions of 'perform' in a generic function

fn perform<F>(world: &mut World, actor_entity: Entity, act: F) -> Milliseconds
where
    F: Fn(Actor) -> Milliseconds,
{
    let mut system_state = SystemState::<(Actors,)>::new(world);
    let (actors,) = system_state.get_mut(world);
    let duration = act(actors.get(actor_entity));
    system_state.apply(world);
    duration
}

fn perform_commands_envir<F>(world: &mut World, actor_entity: Entity, act: F) -> Milliseconds
where
    F: Fn(Actor, &mut Commands, &mut Envir) -> Milliseconds,
{
    let mut system_state = SystemState::<(Commands, Envir, Actors)>::new(world);
    let (mut commands, mut envir, actors) = system_state.get_mut(world);
    let duration = act(actors.get(actor_entity), &mut commands, &mut envir);
    system_state.apply(world);
    duration
}

fn perform_commands_location_hierarchy<F>(
    world: &mut World,
    actor_entity: Entity,
    act: F,
) -> Milliseconds
where
    F: Fn(Actor, &mut Commands, &mut Location, &Hierarchy) -> Milliseconds,
{
    let mut system_state =
        SystemState::<(Commands, Envir, Hierarchy, Actors<'static, 'static>)>::new(world);
    let (mut commands, mut envir, hierarchy, actors) = system_state.get_mut(world);
    let duration = act(
        actors.get(actor_entity),
        &mut commands,
        &mut envir.location,
        &hierarchy,
    );
    system_state.apply(world);
    duration
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_timeouts(
    In(option): In<Option<(Entity, Milliseconds)>>,
    mut commands: Commands,
    mut timeouts: ResMut<Timeouts>,
    players: Query<&Player>,
) {
    let start = Instant::now();

    if let Some((actor_entity, mut timeout)) = option {
        if timeout.0 == 0 && players.get(actor_entity).is_err() {
            commands.spawn(Message::error("failed npc action"));
            timeout.0 = 1000;
        }

        timeouts.add(actor_entity, timeout);
    }

    log_if_slow("manage_characters", start);
}
