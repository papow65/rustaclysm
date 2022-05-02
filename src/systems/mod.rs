mod check;
mod input;
mod startup;
mod update;

use bevy::prelude::*;
use std::time::{Duration, Instant};

use super::components::{Appearance, Health, Label, Player, Pos};
use super::resources::{Characters, Envir, Hierarchy, Instructions, Spawner, Timeouts};

pub use check::*;
pub use input::*;
pub use startup::*;
pub use update::*;

fn log_if_slow(name: &str, start: Instant) {
    let duration = Instant::now() - start;
    if Duration::new(0, 200_000) < duration {
        println!("slow system: {name} took {duration:?}");
    }
}

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
    mut instructions: ResMut<Instructions>,
    mut timeouts: ResMut<Timeouts>,
    characters: Characters,
    mut players: Query<&mut Player>,
    dumpees: Query<(Entity, &Parent, &Label)>,
    hierarchy: Hierarchy, // pickup
) {
    let start = Instant::now();

    let entities = characters.c.iter().map(|(e, _, _, _, _, _, _)| e);
    if let Some(character) = timeouts.next(entities) {
        let factions = characters.collect_factions();
        let (entity, label, &pos, &speed, health, faction, container) =
            characters.c.get(character).unwrap();
        let action = if let Ok(ref mut player) = players.get_mut(entity) {
            if let Some(instruction) = instructions.queue.pop() {
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
        let timeout = action.perform(
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
        assert!(
            players.get(entity).is_ok() || 0 < timeout.0,
            "invalid action fot an npc"
        );

        timeouts.add(character, timeout);
    }

    log_if_slow("manage_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_nearby_zones(
    envir: Envir,
    mut spawner: Spawner,
    moved_players: Query<&Pos, (With<Player>, Changed<Pos>)>,
) {
    if let Ok(&player_pos) = moved_players.get_single() {
        let groud_pos = Pos(player_pos.0, 0, player_pos.2);
        for offset in [Pos(-1, 0, 0), Pos(1, 0, 0), Pos(0, 0, -1), Pos(0, 0, 1)] {
            if let Some(nbor) = groud_pos.nbor(offset) {
                if !envir.has_floor(nbor) {
                    spawner.load_cdda_region(Pos(24 * (nbor.0 / 24), 0, 24 * (nbor.2 / 24)), 1);
                }
            }
        }
    }
}
