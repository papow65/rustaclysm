use crate::prelude::{
    HiddenElevation, Instruction, InstructionQueue, KeyCombo, ManualDisplay, Player,
    RefreshVisualizations, ZoomDirection,
};
use bevy::{
    ecs::event::Events,
    input::{keyboard::KeyboardInput, mouse::MouseWheel, ButtonState},
    prelude::{
        Commands, Entity, EventReader, Input, KeyCode, Query, Res, ResMut, Visibility, With,
    },
};
use std::time::Instant;

use super::log_if_slow;

fn quit(app_exit_events: &mut Events<bevy::app::AppExit>) {
    app_exit_events.send(bevy::app::AppExit);
}

fn zoom(player: &mut Query<&mut Player>, direction: ZoomDirection) {
    //println!("{direction:?}");
    player.single_mut().camera_distance *= 0.75_f32.powi(if direction == ZoomDirection::In {
        1
    } else {
        -1
    });
}

fn toggle_elevation(
    commands: &mut Commands,
    hidden_elevation: &Query<Entity, With<HiddenElevation>>,
) {
    if let Ok(hidden_elevation) = hidden_elevation.get_single() {
        commands.entity(hidden_elevation).despawn();
    } else {
        commands.spawn(HiddenElevation);
    };
    commands.spawn(RefreshVisualizations);
}

fn toggle_help(help: &mut Query<&mut Visibility, With<ManualDisplay>>) {
    for mut visibility in help.iter_mut() {
        *visibility = if *visibility == Visibility::Hidden {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_mouse_input(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut player: Query<&mut Player>,
) {
    let start = Instant::now();
    for scroll_event in mouse_wheel_events.iter() {
        zoom(
            &mut player,
            if 0.0 < scroll_event.y {
                ZoomDirection::In
            } else {
                ZoomDirection::Out
            },
        );
    }

    log_if_slow("manage_mouse_input", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_keyboard_input(
    mut commands: Commands,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut key_events: EventReader<KeyboardInput>,
    mut instruction_queue: ResMut<InstructionQueue>,
    keys: Res<Input<KeyCode>>,
    mut player: Query<&mut Player>,
    hidden_elevation: Query<Entity, With<HiddenElevation>>,
    mut help: Query<&mut Visibility, With<ManualDisplay>>,
) {
    let start = Instant::now();

    for key_event in key_events.iter() {
        let combo = KeyCombo::new(key_event, &keys);
        //println!("{:?} -> {}", &key_event, &combo);
        if let Ok(instruction) = Instruction::try_from(&combo) {
            match key_event.state {
                ButtonState::Pressed => {
                    println!("{:?} -> {} -> {:?}", &key_event, &combo, &instruction);
                    match instruction {
                        Instruction::Quit => quit(&mut app_exit_events),
                        Instruction::Zoom(direction) => zoom(&mut player, direction),
                        Instruction::ToggleElevation => {
                            toggle_elevation(&mut commands, &hidden_elevation);
                        }
                        Instruction::ToggleHelp => toggle_help(&mut help),
                        Instruction::Queued(instruction) => instruction_queue.add(instruction),
                    }
                }
                ButtonState::Released => {
                    if let Instruction::Queued(queued) = instruction {
                        instruction_queue.interrupt(queued);
                    }
                }
            }
        }
    }

    instruction_queue.log_if_long();

    log_if_slow("manage_keyboard_input", start);
}
