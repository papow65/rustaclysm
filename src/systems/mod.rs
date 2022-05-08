mod check;
mod hud;
mod input;
mod startup;
mod update;

use bevy::prelude::*;
use std::time::{Duration, Instant};

use super::components::{Appearance, Health, Label, Message, Player, Pos, Zone, ZoneChanged};
use super::resources::{Characters, Envir, Hierarchy, Instructions, TileSpawner, Timeouts};

pub use check::*;
pub use hud::*;
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

    let entities = characters
        .c
        .iter()
        .map(|(e, _, pos, _, _, _, _)| (e, pos))
        .filter(|(e, &pos)| envir.has_floor(pos) || players.get(*e).is_ok())
        .map(|(e, _)| e);
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

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_nearby_zones(
    mut commands: Commands,
    envir: Envir,
    mut tile_spawner: TileSpawner,
    moved_players: Query<(Entity, &Pos), (With<Player>, With<ZoneChanged>)>,
) {
    if let Ok((player, &pos)) = moved_players.get_single() {
        for zone in Zone::from(pos).nearby(3) {
            if !envir.has_floor(zone.zone_level(0).base_pos()) {
                commands.entity(player).remove::<ZoneChanged>();
                //commands.spawn().insert(Message::new("Spawning zone"));
                tile_spawner.load_cdda_region(zone, 1);
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn despawn_far_zones(
    mut commands: Commands,
    zones: Query<(Entity, &Zone)>,
    moved_players: Query<&Pos, (With<Player>, With<ZoneChanged>)>,
) {
    if let Ok(&pos) = moved_players.get_single() {
        let player_zone = Zone::from(pos);
        //println!("Current zone: {:?}", player_zone);
        for (entity, zone) in zones.iter() {
            //println!("{:?} <-> {:?} : {}", entity, zone, zone.dist(player_zone));
            if 3 < zone.dist(player_zone) {
                //println!("Despawning zone");
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
