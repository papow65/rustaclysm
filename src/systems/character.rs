use super::log_if_slow;
use crate::prelude::*;
use bevy::prelude::*;
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub fn manage_game_over(
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
pub fn manage_characters(
    mut commands: Commands,
    mut envir: Envir,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut timeouts: ResMut<Timeouts>,
    characters: Characters,
    mut players: Query<&mut Player>,
    dumpees: Query<(Entity, &Parent, &Label)>,
    hierarchy: Hierarchy, // pickup
) {
    let start = Instant::now();

    let entities = characters
        .c
        .iter()
        .map(|(e, _, pos, _, _, _, _)| (e, pos))
        .filter(|(e, &pos)| envir.has_floor(pos) || players.get(*e).is_ok())
        .map(|(e, _)| e)
        .collect::<Vec<Entity>>();
    if let Some(character) = timeouts.next(&entities) {
        let factions = characters.collect_factions();
        let (entity, label, &pos, &speed, health, faction, container) =
            characters.c.get(character).unwrap();
        let action = if let Ok(ref mut player) = players.get_mut(entity) {
            if let Some(instruction) = instruction_queue.pop() {
                match player.behave(&envir, pos, instruction) {
                    Ok(action) => action,
                    Err(Some(message)) => {
                        commands.spawn().insert(message);
                        return; // invalid instruction - wait for the user
                    }
                    Err(None) => {
                        return; // valid instruction, but no action performed
                    }
                }
            } else {
                return; // no key pressed - wait for the user
            }
        } else {
            let strategy = faction.behave(&envir, pos, speed, health, &factions);
            println!(
                "{} at {:?} plans {:?} and does {:?}",
                &label, pos, strategy.intent, strategy.action
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
            container,
        );

        if timeout.0 == 0 && players.get(character).is_err() {
            commands
                .spawn()
                .insert(Message::new("ERROR: invalid action fot an npc"));
            timeout.0 = 1000;
        }

        timeouts.add(character, timeout);
    }

    log_if_slow("manage_characters", start);
}
