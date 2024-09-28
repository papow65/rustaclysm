use crate::common::log_if_slow;
use crate::gameplay::{
    CameraOffset, ElevationVisibility, Focus, FocusState, GameplayScreenState, Instruction,
    InstructionQueue, MessageWriter, PlayerActionState, QueuedInstruction, VisualizationUpdate,
    ZoomDirection, ZoomDistance,
};
use crate::hud::ScrollingList;
use crate::keyboard::{InputChange, Key, KeyBinding, KeyChange};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::{prelude::*, render::view::RenderLayers};
use std::time::Instant;

fn open_menu(next_gameplay_state: &mut NextState<GameplayScreenState>) {
    next_gameplay_state.set(GameplayScreenState::Menu);
}

fn open_crafting_screen(next_gameplay_state: &mut NextState<GameplayScreenState>) {
    next_gameplay_state.set(GameplayScreenState::Crafting);
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
        camera_layers.clone().with(1).without(2)
    } else {
        camera_offset.zoom_to_map(zoom_distance);
        camera_layers.clone().without(1).with(2)
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
            *camera_layers = camera_layers.clone().with(1).without(2);
        }
    } else if camera_offset.zoom_map_only() {
        *camera_layers = camera_layers.clone().without(1).with(2);
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

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_mouse_scroll_input(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_offset: ResMut<CameraOffset>,
    mut camera_layers: Query<&mut RenderLayers, With<Camera3d>>,
    scrolling_lists: Query<&Interaction, With<ScrollingList>>,
) {
    let start = Instant::now();

    if scrolling_lists.iter().all(|i| i == &Interaction::None) {
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
    }

    log_if_slow("manage_mouse_scroll_input", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_mouse_button_input(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera_offset: ResMut<CameraOffset>,
) {
    let start = Instant::now();

    if mouse_buttons.pressed(MouseButton::Middle) {
        let delta_sum = mouse_motion_events
            .read()
            .map(|motion_event| motion_event.delta)
            .sum();
        camera_offset.adjust_angle(delta_sum);
    }

    log_if_slow("manage_mouse_button_input", start);
}

fn handle_queued_instruction(
    message_writer: &mut MessageWriter,
    focus_state: &FocusState,
    next_focus_state: &mut ResMut<NextState<FocusState>>,
    next_player_action_state: &mut ResMut<NextState<PlayerActionState>>,
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
            next_player_action_state.set(PlayerActionState::AutoTravel { target });
            instruction_queue.stop_waiting();
            message_writer.you("start traveling...").send_info();
        }
        (FocusState::ExaminingZoneLevel(zone_level), QueuedInstruction::ToggleAutoTravel) => {
            //println!("Autotravel zone level");
            next_focus_state.set(FocusState::Normal);
            next_player_action_state.set(PlayerActionState::AutoTravel {
                target: zone_level.center_pos(),
            });
            instruction_queue.stop_waiting();
            message_writer.you("start traveling...").send_info();
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

pub(super) fn create_base_key_bindings(world: &mut World) {
    let start = Instant::now();

    let manage_held_keyboard_input =
        world.register_system(to_held_instruction.pipe(manage_keyboard_input));

    world.spawn_batch([
        KeyBinding::from_multi(
            [
                KeyCode::Numpad1,
                KeyCode::Numpad2,
                KeyCode::Numpad3,
                KeyCode::Numpad4,
                KeyCode::Numpad5,
                KeyCode::Numpad6,
                KeyCode::Numpad7,
                KeyCode::Numpad8,
                KeyCode::Numpad9,
            ],
            manage_held_keyboard_input,
        )
        .held()
        .scoped(GameplayScreenState::Base),
        KeyBinding::from_multi(['<', '>'], manage_held_keyboard_input)
            .held()
            .scoped(GameplayScreenState::Base),
    ]);

    let manage_fresh_keyboard_input =
        world.register_system(to_fresh_instruction.pipe(manage_keyboard_input));
    world.spawn_batch([
        KeyBinding::from_multi([KeyCode::Escape, KeyCode::Tab], manage_fresh_keyboard_input)
            .scoped(GameplayScreenState::Base),
        KeyBinding::from_multi(
            [
                'm', 'M', 'x', 'X', '&', 'i', 'Z', 'z', 'h', '0', '|', '$', 'a', 's', 'p', 'c',
                '\\', 'G', 'A', '+',
            ],
            manage_fresh_keyboard_input,
        )
        .scoped(GameplayScreenState::Base),
    ]);

    log_if_slow("create_crafting_key_bindings", start);
}

#[expect(clippy::needless_pass_by_value)]
fn to_held_instruction(
    In(key): In<Key>,
    focus: Focus,
    player_action_state: Res<State<PlayerActionState>>,
) -> (Instruction, InputChange) {
    let key_change = KeyChange {
        key,
        change: InputChange::Held,
    }; // FIXME not correct

    (
        Instruction::try_from((
            key_change,
            focus.state.cancel_handling(&player_action_state),
        ))
        .expect("Conversion failed for held instruction"),
        key_change.change,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn to_fresh_instruction(
    In(key): In<Key>,
    focus: Focus,
    player_action_state: Res<State<PlayerActionState>>,
) -> (Instruction, InputChange) {
    let key_change = KeyChange {
        key,
        change: InputChange::JustPressed,
    };

    (
        Instruction::try_from((
            key_change,
            focus.state.cancel_handling(&player_action_state),
        ))
        .expect("Conversion failed for fresh instruction"),
        key_change.change,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn manage_keyboard_input(
    In((instruction, input_change)): In<(Instruction, InputChange)>,
    mut message_writer: MessageWriter,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    focus: Focus,
    mut next_focus_state: ResMut<NextState<FocusState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut elevation_visibility: ResMut<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut camera_offset: ResMut<CameraOffset>,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    mut camera_layers: Query<&mut RenderLayers, With<Camera3d>>,
) {
    let start = Instant::now();

    println!("'-> {:?}", &instruction);
    match instruction {
        Instruction::ShowGameplayMenu => open_menu(&mut next_gameplay_state),
        Instruction::ExaminePos => {
            focus.toggle_examine_pos(&mut next_focus_state);
        }
        Instruction::ExamineZoneLevel => {
            focus.toggle_examine_zone_level(&mut next_focus_state);
        }
        Instruction::CraftingScreen => open_crafting_screen(&mut next_gameplay_state),
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
            &focus.state,
            &mut next_focus_state,
            &mut next_player_action_state,
            &mut instruction_queue,
            instruction,
            input_change,
        ),
    }

    instruction_queue.log_if_long();

    log_if_slow("manage_keyboard_input", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn update_camera_offset(
    camera_offset: Res<CameraOffset>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let start = Instant::now();

    let mut transform = cameras.single_mut();
    transform.translation = camera_offset.offset();
    transform.look_at(Vec3::ZERO, Vec3::Y);

    log_if_slow("update_camera", start);
}

pub(in super::super) fn trigger_refresh(mut visualization_update: ResMut<VisualizationUpdate>) {
    *visualization_update = VisualizationUpdate::Forced;
}
