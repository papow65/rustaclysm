use bevy::ecs::event::Events;
use bevy::input::{keyboard::KeyboardInput, mouse::MouseWheel, ButtonState};
use bevy::prelude::{EventReader, Input, KeyCode, Local, Query, Res, ResMut, Visibility, With};
use std::time::Instant;

use crate::components::{Instruction, ManualDisplay, Player};
use crate::resources::Instructions;

use super::log_if_slow;

fn zoom(player: &mut Player, in_: bool) {
    player.camera_distance *= 0.75_f32.powi(if in_ { 1 } else { -1 });
}

#[allow(clippy::needless_pass_by_value)]
pub fn manage_mouse_input(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut player: Query<&mut Player>,
) {
    let start = Instant::now();

    let mut player = player.iter_mut().next().unwrap();

    for scroll_event in mouse_wheel_events.iter() {
        zoom(&mut player, 0.0 < scroll_event.y);
    }

    log_if_slow("manage_mouse_input", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn manage_keyboard_input(
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut key_events: EventReader<KeyboardInput>,
    mut instructions: ResMut<Instructions>,
    keys: Res<Input<KeyCode>>,
    mut keys_held: Local<Vec<KeyCode>>,
    mut player: Query<&mut Player>,
    mut help: Query<&mut Visibility, With<ManualDisplay>>,
) {
    let start = Instant::now();

    for key_event in key_events.iter() {
        match key_event.state {
            ButtonState::Pressed => {
                if let Some(key_code) = key_event.key_code {
                    let control =
                        keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl);
                    let shift = keys.pressed(KeyCode::LShift) || keys.pressed(KeyCode::RShift);
                    println!(
                        "{}{}{:?} pressed",
                        if control { "ctrl+" } else { "" },
                        if shift { "shift+" } else { "" },
                        key_code
                    );
                    match (control, shift, key_code) {
                        (true, _, KeyCode::C | KeyCode::D | KeyCode::Q) => {
                            app_exit_events.send(bevy::app::AppExit);
                        }
                        (false, _, KeyCode::Z) => {
                            zoom(&mut player.single_mut(), !shift);
                        }
                        (false, false, KeyCode::H) => {
                            for mut visibility in help.iter_mut() {
                                visibility.is_visible ^= true; // XOR
                            }
                        }
                        (false, false, _) => {
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
                        (..) => {}
                    }
                }
            }
            ButtonState::Released => {
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
