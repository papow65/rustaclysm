use super::log_if_slow;
use crate::prelude::*;
use bevy::prelude::*;
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
pub(crate) fn manage_characters(
    mut commands: Commands,
    mut envir: Envir,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut timeouts: ResMut<Timeouts>,
    characters: Characters,
    mut players: Query<&mut Player>,
    dumpees: Query<(Entity, &Label, &Parent)>,
    hierarchy: Hierarchy, // pickup
) {
    let start = Instant::now();

    while start.elapsed() < MAX_SYSTEM_DURATION / 2 {
        let entities = characters
            .c
            .iter()
            .map(|(e, _, pos, ..)| (e, pos))
            .filter(|(e, &pos)| envir.has_floor(pos) || players.get(*e).is_ok())
            .map(|(e, _)| e)
            .collect::<Vec<Entity>>();
        if let Some(character) = timeouts.next(&entities) {
            let factions = characters.collect_factions();
            let (
                entity,
                label,
                &pos,
                &speed,
                health,
                faction,
                melee,
                hands,
                clothing,
                aquatic,
                last_enemy,
            ) = characters.c.get(character).unwrap();
            let action = if let Ok(ref mut player) = players.get_mut(entity) {
                if let Some(action) = player.plan_action(
                    &mut commands,
                    &mut envir,
                    &mut instruction_queue,
                    pos,
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
                let strategy =
                    faction.behave(&envir, pos, speed, health, aquatic, &factions, last_enemy);
                if let Some(l) = strategy.last_enemy {
                    commands.entity(character).insert(l);
                }
                println!(
                    "{} at {pos:?} plans {:?} and does {:?}",
                    &label, strategy.intent, strategy.action
                );
                strategy.action
            };
            let mut timeout = action.perform(
                &mut commands,
                &mut envir,
                &dumpees,
                &hierarchy,
                character,
                label,
                pos,
                speed,
                melee,
                hands,
                clothing,
            );

            if timeout.0 == 0 && players.get(character).is_err() {
                commands.spawn(Message::error("failed npc action"));
                timeout.0 = 1000;
            }

            timeouts.add(character, timeout);
        } else {
            // No characters!
            break;
        }
    }

    log_if_slow("manage_characters", start);
}
