use crate::prelude::*;
use bevy::{
    app::AppExit,
    ecs::event::Events,
    input::{mouse::MouseWheel, ButtonState},
    prelude::*,
};
use std::time::Instant;

fn quit(app_exit_events: &mut Events<AppExit>) {
    app_exit_events.send(AppExit);
}

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
    mut camera_offset: ResMut<CameraOffset>,
) {
    let start = Instant::now();
    for scroll_event in &mut mouse_wheel_events {
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
    mut keys: Keys,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut elevation_visibility: ResMut<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut camera_offset: ResMut<CameraOffset>,
    player_action_state: Res<PlayerActionState>,
    mut help: Query<&mut Visibility, With<ManualDisplay>>,
) {
    let start = Instant::now();

    for (button_state, combo) in keys.combos() {
        let Ok(instruction) = Instruction::try_from((&combo, player_action_state.cancel_context()))
        else {
            if button_state == ButtonState::Pressed {
                println!("{:?} not recognized", &combo);
            }
            continue;
        };
        match button_state {
            ButtonState::Pressed => {
                println!("{:?} -> {:?}", &combo, &instruction);
                handle_instruction(
                    instruction,
                    &mut app_exit_events,
                    &mut next_application_state,
                    &mut next_gameplay_state,
                    &mut camera_offset,
                    &mut elevation_visibility,
                    &mut visualization_update,
                    &mut help,
                    &mut instruction_queue,
                );
            }
            ButtonState::Released => {
                if let Instruction::Queued(queued) = instruction {
                    instruction_queue.interrupt(&queued);
                }
            }
        }
    }

    instruction_queue.log_if_long();

    log_if_slow("manage_keyboard_input", start);
}

fn handle_instruction(
    instruction: Instruction,
    app_exit_events: &mut ResMut<Events<AppExit>>,
    next_application_state: &mut ResMut<NextState<ApplicationState>>,
    next_gameplay_state: &mut ResMut<NextState<GameplayScreenState>>,
    camera_offset: &mut ResMut<CameraOffset>,
    elevation_visibility: &mut ResMut<ElevationVisibility>,
    visualization_update: &mut ResMut<VisualizationUpdate>,
    help: &mut Query<&mut Visibility, With<ManualDisplay>>,
    instruction_queue: &mut ResMut<InstructionQueue>,
) {
    match instruction {
        Instruction::Quit => quit(app_exit_events),
        Instruction::MainMenu => {
            open_main_menu(next_application_state, next_gameplay_state);
        }
        Instruction::CancelState => open_menu(next_gameplay_state),
        Instruction::Inventory => open_inventory(next_gameplay_state),
        Instruction::Zoom(direction) => zoom(camera_offset, direction),
        Instruction::ToggleElevation => {
            toggle_elevation(elevation_visibility, visualization_update);
        }
        Instruction::ToggleHelp => toggle_help(help),
        Instruction::Queued(instruction) => instruction_queue.add(instruction),
    }
}
