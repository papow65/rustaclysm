use crate::prelude::*;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use std::time::Instant;

fn open_main_menu(
    next_application_state: &mut NextState<ApplicationState>,
    next_gameplay_state: &mut NextState<GameplayScreenState>,
) {
    next_gameplay_state.set(GameplayScreenState::Inapplicable);
    next_application_state.set(ApplicationState::MainMenu);
}

fn open_menu(next_gameplay_state: &mut NextState<GameplayScreenState>) {
    next_gameplay_state.set(GameplayScreenState::Menu);
}

fn open_inventory(next_gameplay_state: &mut NextState<GameplayScreenState>) {
    next_gameplay_state.set(GameplayScreenState::Inventory);
}

fn zoom(camera_offset: &mut CameraOffset, direction: ZoomDirection) {
    camera_offset.zoom(direction);
}

fn reset_camera_angle(camera_offset: &mut CameraOffset) {
    camera_offset.reset_angle();
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
    for mut visibility in &mut *help {
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
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera_offset: ResMut<CameraOffset>,
) {
    let start = Instant::now();
    for scroll_event in &mut mouse_wheel_events.read() {
        zoom(
            &mut camera_offset,
            if 0.0 < scroll_event.y {
                ZoomDirection::In
            } else {
                ZoomDirection::Out
            },
        );
    }

    if mouse_buttons.pressed(MouseButton::Middle) {
        let delta_sum = mouse_motion_events
            .read()
            .map(|motion_event| motion_event.delta)
            .sum();
        camera_offset.adjust_angle(delta_sum);
    }

    log_if_slow("manage_mouse_input", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_keyboard_input(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut keys: Keys,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut elevation_visibility: ResMut<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut camera_offset: ResMut<CameraOffset>,
    player_action_state: Res<PlayerActionState>,
    mut help: Query<&mut Visibility, With<ManualDisplay>>,
) {
    let start = Instant::now();

    for combo in keys.combos(Ctrl::Without) {
        let Ok(instruction) = Instruction::try_from((&combo, player_action_state.cancel_context()))
        else {
            println!("{combo:?} not recognized");
            continue;
        };

        println!("{:?} -> {:?}", &combo, &instruction);
        match instruction {
            Instruction::ShowMainMenu => {
                open_main_menu(&mut next_application_state, &mut next_gameplay_state);
            }
            Instruction::ShowGameplayMenu => open_menu(&mut next_gameplay_state),
            Instruction::Inventory => open_inventory(&mut next_gameplay_state),
            Instruction::Zoom(direction) => zoom(&mut camera_offset, direction),
            Instruction::ResetCameraAngle => reset_camera_angle(&mut camera_offset),
            Instruction::ToggleElevation => {
                toggle_elevation(&mut elevation_visibility, &mut visualization_update);
            }
            Instruction::ToggleHelp => toggle_help(&mut help),
            Instruction::Queued(instruction) => instruction_queue.add(instruction, combo.change),
        }
    }

    instruction_queue.log_if_long();

    log_if_slow("manage_keyboard_input", start);
}
