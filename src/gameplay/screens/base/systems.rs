use crate::prelude::*;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::view::RenderLayers,
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

fn toggle_map(
    camera_offset: &mut CameraOffset,
    camera_layers: &mut Query<&mut RenderLayers, With<Camera3d>>,
    zoom_distance: ZoomDistance,
) {
    let mut camera_layers = camera_layers.single_mut();
    *camera_layers = if showing_map(&camera_layers) {
        camera_offset.zoom_to_tiles(zoom_distance);
        camera_layers.with(1).without(2)
    } else {
        camera_offset.zoom_to_map(zoom_distance);
        camera_layers.without(1).with(2)
    };
}

fn zoom(
    camera_offset: &mut CameraOffset,
    camera_layers: &mut Query<&mut RenderLayers, With<Camera3d>>,
    direction: ZoomDirection,
) {
    camera_offset.zoom(direction);

    let mut camera_layers = camera_layers.single_mut();
    if showing_map(&camera_layers) {
        if camera_offset.zoom_tiles_only() {
            *camera_layers = camera_layers.with(1).without(2);
        }
    } else if camera_offset.zoom_map_only() {
        *camera_layers = camera_layers.without(1).with(2);
    }
}

fn showing_map(camera_layers: &RenderLayers) -> bool {
    camera_layers.intersects(&RenderLayers::layer(2))
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
    mut camera_layers: Query<&mut RenderLayers, With<Camera3d>>,
) {
    let start = Instant::now();
    for scroll_event in &mut mouse_wheel_events.read() {
        zoom(
            &mut camera_offset,
            &mut camera_layers,
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
    mut camera_layers: Query<&mut RenderLayers, With<Camera3d>>,
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
            Instruction::ToggleMap(zoom_distance) => {
                toggle_map(&mut camera_offset, &mut camera_layers, zoom_distance);
            }
            Instruction::Zoom(direction) => zoom(&mut camera_offset, &mut camera_layers, direction),
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
