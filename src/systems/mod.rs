mod check;
mod input;
mod startup;
mod update;

use bevy::prelude::*;
use bevy::render::camera::Camera;
use std::time::{Duration, Instant};

use super::components::{Action, Appearance, Corpse, Health, Label, Message, Player, Pos, Status};
use super::resources::{Characters, Envir, Hierarchy, Instructions, Timeouts};

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
pub fn manage_status(
    mut commands: Commands,
    debuggers: Query<Entity, With<Status>>,
    all: Query<
        (
            Entity,
            Option<&Label>,
            Option<&Pos>,
            Option<&Action>,
            Option<&Parent>,
            Option<&Children>,
            &GlobalTransform,
        ),
        Or<(With<Health>, With<Corpse>, With<Children>, With<Camera>)>,
    >,
) {
    let start = Instant::now();

    for debugger in debuggers.iter() {
        for (entity, label, pos, action, parent, children, gt) in all.iter() {
            let message = format!(
                "{} | {:?} -> {:?} | {:?} > {:?} > {:?}\nGT: {:?} {:?} {:?}",
                label.unwrap_or(&Label::new("-")),
                pos.unwrap_or(&Pos(9, 9, 9)),
                action,
                parent,
                entity,
                children,
                gt.translation,
                gt.rotation,
                gt.scale
            );
            commands.spawn_bundle(Message::new(message));
        }
        commands.entity(debugger).despawn();
    }

    log_if_slow("manage_status", start);
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
                    Err(messages) => {
                        for message in messages {
                            commands.spawn_bundle(message);
                        }
                        return; // invalid key - wait for the user
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
