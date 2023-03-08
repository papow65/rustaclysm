use super::log_if_slow;
use crate::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*};
use std::time::{Duration, Instant};

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
pub(crate) fn manage_characters(world: &mut World) {
    let start = Instant::now();

    let mut count = 0;
    loop {
        let iteration = Instant::now();

        let mut system_state: SystemState<(
            Commands,
            Envir,
            ResMut<InstructionQueue>,
            ResMut<Timeouts>,
            Actors,
            Query<&mut Player>,
            Query<(Entity, &TextLabel, &Parent)>,
            Hierarchy, // pickup
        )> = SystemState::new(world);

        let (
            mut commands,
            mut envir,
            mut instruction_queue,
            mut timeouts,
            actors,
            mut players,
            dumpees,
            hierarchy,
        ) = system_state.get_mut(world);

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
                    break; // process the cancellation next turn
                } else {
                    Action::Stay
                }
            } else {
                break; // no key pressed - wait for the user
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
        println!(
            "iteration of manage characters for {:?} took {:?} ({:?} since start)",
            actor.label,
            iteration.elapsed(),
            start.elapsed(),
        );

        system_state.apply(world);

        count += 1;
        println!(
            "iteration of manage characters took {:?} after appling ({:?} since start)",
            iteration.elapsed(),
            start.elapsed(),
        );
        if Duration::from_millis(2) * 3 / 4 < start.elapsed() {
            eprintln!("manage_characters could ony handle {count} iterations before the timeout");
            break;
        }
    }

    log_if_slow("manage_characters", start);
}
