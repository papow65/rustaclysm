use crate::prelude::*;
use bevy::{
    app::AppExit,
    ecs::event::Events,
    input::{keyboard::KeyboardInput, mouse::MouseWheel, ButtonState},
    prelude::*,
};
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_base_resources(mut commands: Commands) {
    commands.insert_resource(InstructionQueue::default());
}

fn quit(app_exit_events: &mut Events<AppExit>) {
    app_exit_events.send(AppExit);
}

fn main_menu(
    next_application_state: &mut NextState<ApplicationState>,
    next_gameplay_state: &mut NextState<GameplayScreenState>,
) {
    next_gameplay_state.set(GameplayScreenState::Inapplicable);
    next_application_state.set(ApplicationState::MainMenu);
}

fn zoom(camera_offset: &mut CameraOffset, direction: ZoomDirection) {
    camera_offset.zoom(direction);
}

fn toggle_elevation(
    elevation_visiblity: &mut ElevationVisibility,
    visualization_update: &mut VisualizationUpdate,
) {
    *elevation_visiblity = match elevation_visiblity {
        ElevationVisibility::Shown => ElevationVisibility::Hidden,
        ElevationVisibility::Hidden => ElevationVisibility::Shown,
    };
    *visualization_update = VisualizationUpdate::Forced;
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
    mut camera_offset: ResMut<CameraOffset>,
) {
    let start = Instant::now();
    for scroll_event in mouse_wheel_events.iter() {
        zoom(
            &mut camera_offset,
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
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut key_events: EventReader<KeyboardInput>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut elevation_visibility: ResMut<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut camera_offset: ResMut<CameraOffset>,
    keys: Res<Input<KeyCode>>,
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
                        Instruction::MainMenu => {
                            main_menu(&mut next_application_state, &mut next_gameplay_state);
                        }
                        Instruction::Zoom(direction) => zoom(&mut camera_offset, direction),
                        Instruction::ToggleElevation => {
                            toggle_elevation(&mut elevation_visibility, &mut visualization_update);
                        }
                        Instruction::ToggleHelp => toggle_help(&mut help),
                        Instruction::Queued(instruction) => instruction_queue.add(instruction),
                    }
                }
                ButtonState::Released => {
                    if let Instruction::Queued(queued) = instruction {
                        instruction_queue.interrupt(&queued);
                    }
                }
            }
        }
    }

    instruction_queue.log_if_long();

    log_if_slow("manage_keyboard_input", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_base_resources(mut commands: Commands) {
    commands.remove_resource::<InstructionQueue>();
}
