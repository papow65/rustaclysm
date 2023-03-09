use super::log_if_slow;
use crate::prelude::*;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use std::time::Instant;

// A label for our new Schedule!
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct BehaviorSchedule;

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
pub(crate) fn manage_characters(
    mut commands: Commands,
    mut envir: Envir,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut timeouts: ResMut<Timeouts>,
    actors: Actors,
    mut players: Query<&mut Player>,
    dumpees: Query<(Entity, &TextLabel, &Parent)>,
    hierarchy: Hierarchy, // pickup
) {
    let start = Instant::now();

    let egible_entities = actors
        .actors()
        .filter(|a| envir.is_accessible(a.pos) || players.get(a.entity).is_ok())
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    let Some(active_entity) = timeouts.next(&egible_entities) else {
        eprintln!("No egible characters!");
        return;
    };
    let factions = actors.collect_factions();
    let actor = Actor::from(actors.q.get(active_entity).unwrap());
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
                return; // process the cancellation next turn
            } else {
                Action::Stay
            }
        } else {
            instruction_queue.start_waiting();
            println!("Waiting for user action");
            return; // no key pressed - wait for the user
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
    let mut timeout = actor.perform(&mut commands, &mut envir, &dumpees, &hierarchy, &action);

    if timeout.0 == 0 && players.get(active_entity).is_err() {
        commands.spawn(Message::error("failed npc action"));
        timeout.0 = 1000;
    }

    timeouts.add(active_entity, timeout);

    log_if_slow("manage_characters", start);
}
