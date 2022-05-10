mod check;
mod hud;
mod input;
mod startup;
mod update;

use bevy::prelude::*;
use std::time::{Duration, Instant};

use super::components::{
    Appearance, Health, Label, Message, Player, PlayerActionState, Pos, Zone, ZoneChanged,
};
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
        .map(|(e, _)| e)
        .collect::<Vec<Entity>>();
    if let Some(character) = timeouts.next(&entities) {
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

const SPAWN_DISTANCE: u32 = 3;
const DESPAWN_DISTANCE: u32 = SPAWN_DISTANCE + 1;

fn get_center_zones(pos: Pos, player: &Player) -> Vec<Zone> {
    let mut positions = vec![pos];
    if let PlayerActionState::Examining(camera_pos) = player.state {
        positions.push(camera_pos);
    }
    positions
        .iter()
        .map(|&p| Zone::from(p))
        .collect::<Vec<Zone>>()
}

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_nearby_zones(
    mut commands: Commands,
    envir: Envir,
    mut tile_spawner: TileSpawner,
    players: Query<(Entity, &Pos, &Player), With<ZoneChanged>>,
) {
    // ZoneChanged is set when moving of examining

    if let Ok((entity, &pos, player)) = players.get_single() {
        for center_zone in get_center_zones(pos, player) {
            for nearby_zone in center_zone.nearby(SPAWN_DISTANCE) {
                if !envir.has_floor(nearby_zone.zone_level(0).base_pos()) {
                    tile_spawner.load_cdda_region(nearby_zone, 1);
                }
            }
        }

        // This is applied at the end of the stage,
        // so spawning and despawing should be in he same stage.
        commands.entity(entity).remove::<ZoneChanged>();
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn despawn_far_zones(
    mut commands: Commands,
    checked_zones: Query<(Entity, &Zone)>,
    players: Query<(&Pos, &Player), With<ZoneChanged>>,
) {
    // ZoneChanged is set when moving of examining

    if let Ok((&pos, player)) = players.get_single() {
        let centers = get_center_zones(pos, player);
        let is_far_away = |zone: Zone| {
            centers
                .iter()
                .map(|&center| zone.dist(center))
                .all(|dist_from_center| DESPAWN_DISTANCE <= dist_from_center)
        };
        checked_zones
            .iter()
            .filter(|(_, &checked_zone)| is_far_away(checked_zone))
            .map(|(e, _)| e)
            .for_each(|e| commands.entity(e).despawn_recursive());
    }
}
