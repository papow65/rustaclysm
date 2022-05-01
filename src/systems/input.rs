use bevy::ecs::event::Events;
use bevy::input::{keyboard::KeyboardInput, mouse::MouseWheel, ElementState};
use bevy::prelude::{Commands, EventReader, Input, KeyCode, Local, Query, Res, ResMut};
use std::time::Instant;

use super::super::components::{Instruction, Player, Status};
use super::super::resources::Instructions;

use super::log_if_slow;

#[allow(clippy::needless_pass_by_value)]
pub fn manage_mouse_input(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut player: Query<&mut Player>,
) {
    let start = Instant::now();

    let mut player = player.iter_mut().next().unwrap();

    for scroll_event in mouse_wheel_events.iter() {
        player.camera_distance *= 0.75_f32.powi(scroll_event.y.signum() as i32);
    }

    log_if_slow("manage_mouse_input", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn manage_keyboard_input(
    mut commands: Commands,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut key_events: EventReader<KeyboardInput>,
    mut instructions: ResMut<Instructions>,
    keys: Res<Input<KeyCode>>,
    mut keys_held: Local<Vec<KeyCode>>,
) {
    let start = Instant::now();

    for key_event in key_events.iter() {
        match key_event.state {
            ElementState::Pressed => {
                if let Some(key_code) = key_event.key_code {
                    let control =
                        keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl);
                    println!(
                        "{}{:?} pressed",
                        if control { "ctrl+" } else { "" },
                        key_code
                    );
                    match (key_code, control) {
                        (KeyCode::Escape, _) | (KeyCode::C | KeyCode::D, true) => {
                            app_exit_events.send(bevy::app::AppExit);
                        }
                        (KeyCode::Comma, false) => {
                            commands.spawn_bundle((Status,));
                        }
                        (_, false) => {
                            if let Some(instruction) = Instruction::new(key_code) {
                                // Wait for an instruction to be processed until adding a duplicate when holding a key down.
                                if !keys_held.contains(&key_code)
                                    || !instructions.queue.contains(&instruction)
                                {
                                    instructions.queue.insert(0, instruction);
                                }

                                // Duplicates in 'keys_held' are ignored and fixed on key release.
                                keys_held.push(key_code);
                            }
                        }
                        (_, true) => {}
                    }
                }
            }
            ElementState::Released => {
                if let Some(released_key) = key_event.key_code {
                    keys_held.retain(|k| *k != released_key);
                }
            }
        }
    }

    if 1 < instructions.queue.len() {
        println!("Unprocessed key codes: {:?}", instructions.queue);
    }

    log_if_slow("manage_keyboard_input", start);
}
