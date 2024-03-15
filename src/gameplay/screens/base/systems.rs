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

fn toggle_examine_pos(
    focus_state: &FocusState,
    next_focus_state: &mut NextState<FocusState>,
    players: &Query<&Pos, With<Player>>,
) {
    next_focus_state.set(match focus_state {
        FocusState::Normal => FocusState::ExaminingPos(*players.single()),
        FocusState::ExaminingPos(_) => FocusState::Normal,
        FocusState::ExaminingZoneLevel(zone_level) => {
            FocusState::ExaminingPos(zone_level.center_pos())
        }
    });
}

fn toggle_examine_zone_level(
    focus_state: &FocusState,
    next_focus_state: &mut NextState<FocusState>,
    players: &Query<&Pos, With<Player>>,
) {
    next_focus_state.set(match focus_state {
        FocusState::Normal => FocusState::ExaminingZoneLevel(ZoneLevel::from(*players.single())),
        FocusState::ExaminingPos(pos) => FocusState::ExaminingZoneLevel(ZoneLevel::from(*pos)),
        FocusState::ExaminingZoneLevel(_) => FocusState::Normal,
    });
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

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_mouse_input(
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

fn handle_queued_instruction(
    message_writer: &mut EventWriter<Message>,
    focus_state: &FocusState,
    next_focus_state: &mut ResMut<NextState<FocusState>>,
    player_action_state: &mut ResMut<PlayerActionState>,
    instruction_queue: &mut ResMut<InstructionQueue>,
    instruction: QueuedInstruction,
    change: InputChange,
) {
    //println!("{focus_state:?} {instruction:?}");
    match (*focus_state, &instruction) {
        (FocusState::Normal, _) => instruction_queue.add(instruction, change),
        (FocusState::ExaminingPos(target), QueuedInstruction::ToggleAutoTravel) => {
            //println!("Autotravel pos");
            next_focus_state.set(FocusState::Normal);
            **player_action_state = PlayerActionState::AutoTravel { target };
            instruction_queue.stop_waiting();
            message_writer.send(Message::info(Phrase::new("You start traveling...")));
        }
        (FocusState::ExaminingZoneLevel(zone_level), QueuedInstruction::ToggleAutoTravel) => {
            //println!("Autotravel zone level");
            next_focus_state.set(FocusState::Normal);
            **player_action_state = PlayerActionState::AutoTravel {
                target: zone_level.center_pos(),
            };
            instruction_queue.stop_waiting();
            message_writer.send(Message::info(Phrase::new("You start traveling...")));
        }
        (FocusState::ExaminingPos(target), QueuedInstruction::Offset(offset)) => {
            if let Some(nbor_target) = target.raw_nbor(offset.to_nbor()) {
                next_focus_state.set(FocusState::ExaminingPos(nbor_target));
            }
        }
        (FocusState::ExaminingZoneLevel(target), QueuedInstruction::Offset(offset)) => {
            if let Some(nbor_target) = target.nbor(offset.to_nbor()) {
                next_focus_state.set(FocusState::ExaminingZoneLevel(nbor_target));
            }
        }
        (_, QueuedInstruction::CancelAction) => next_focus_state.set(FocusState::Normal),
        _ => {
            println!("Ignoring {:?} in {:?}", &instruction, &focus_state);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_keyboard_input(
    mut message_writer: EventWriter<Message>,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    focus_state: Res<State<FocusState>>,
    mut next_focus_state: ResMut<NextState<FocusState>>,
    mut keys: Keys,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut elevation_visibility: ResMut<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut camera_offset: ResMut<CameraOffset>,
    mut player_action_state: ResMut<PlayerActionState>,
    mut camera_layers: Query<&mut RenderLayers, With<Camera3d>>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    for combo in keys.combos(Ctrl::Without) {
        let Ok(instruction) =
            Instruction::try_from((&combo, focus_state.cancel_handling(&player_action_state)))
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
            Instruction::ExaminePos => {
                toggle_examine_pos(&focus_state, &mut next_focus_state, &players);
            }
            Instruction::ExamineZoneLevel => {
                toggle_examine_zone_level(&focus_state, &mut next_focus_state, &players);
            }
            Instruction::Inventory => open_inventory(&mut next_gameplay_state),
            Instruction::ToggleMap(zoom_distance) => {
                toggle_map(&mut camera_offset, &mut camera_layers, zoom_distance);
            }
            Instruction::Zoom(direction) => zoom(&mut camera_offset, &mut camera_layers, direction),
            Instruction::ResetCameraAngle => reset_camera_angle(&mut camera_offset),
            Instruction::ToggleElevation => {
                toggle_elevation(&mut elevation_visibility, &mut visualization_update);
            }
            Instruction::Queued(instruction) => handle_queued_instruction(
                &mut message_writer,
                &focus_state,
                &mut next_focus_state,
                &mut player_action_state,
                &mut instruction_queue,
                instruction,
                combo.change,
            ),
        }
    }

    instruction_queue.log_if_long();

    log_if_slow("manage_keyboard_input", start);
}
