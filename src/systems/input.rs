use bevy::app::Events;
use bevy::input::{keyboard::KeyboardInput, mouse::MouseWheel};
use bevy::prelude::{Commands, EventReader, Input, KeyCode, Query, Res, ResMut};
use std::time::Instant;

use super::super::components::*;
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
) {
    let start = Instant::now();

    for key_event in key_events.iter() {
        if key_event.state.is_pressed() {
            if let Some(key_code) = key_event.key_code {
                let control = keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl);
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
                            instructions.queue.insert(0, instruction);
                        }
                    }
                    (_, true) => {}
                }
            }
        }
    }

    if 1 < instructions.queue.len() {
        println!("Unprocessed key codes: {:?}", instructions.queue);
    }

    log_if_slow("manage_keyboard_input", start);
}
