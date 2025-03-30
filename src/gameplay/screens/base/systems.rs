use crate::gameplay::{
    CameraOffset, CancelHandling, ChangePace, ElevationVisibility, Focus, FocusState,
    GameplayScreenState, InstructionQueue, MessageWriter, PlayerActionState, PlayerDirection,
    QueuedInstruction, VisualizationUpdate, ZoomDirection, ZoomDistance,
};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::{prelude::*, render::view::RenderLayers};
use hud::ScrollList;
use keyboard::{Held, KeyBindings};
use manual::ManualSection;
use std::time::Instant;
use strum::VariantArray as _;
use util::log_if_slow;

#[expect(clippy::needless_pass_by_value)]
fn examine_pos(focus: Focus, mut next_focus_state: ResMut<NextState<FocusState>>) {
    let start = Instant::now();

    focus.toggle_examine_pos(&mut next_focus_state);

    log_if_slow("examine_pos", start);
}

#[expect(clippy::needless_pass_by_value)]
fn examine_zone_level(focus: Focus, mut next_focus_state: ResMut<NextState<FocusState>>) {
    let start = Instant::now();

    focus.toggle_examine_zone_level(&mut next_focus_state);

    log_if_slow("examine_zone_level", start);
}

fn open_crafting_screen(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Crafting);

    log_if_slow("open_crafting_screen", start);
}

fn open_inventory(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Inventory);

    log_if_slow("open_inventory", start);
}

fn toggle_map(
    In(zoom_distance): In<ZoomDistance>,
    mut camera_offset: ResMut<CameraOffset>,
    mut camera_layers: Single<&mut RenderLayers, With<Camera3d>>,
) {
    let start = Instant::now();

    **camera_layers = if showing_map(&camera_layers) {
        camera_offset.zoom_to_tiles(zoom_distance);
        (*camera_layers).clone().with(1).without(2)
    } else {
        camera_offset.zoom_to_map(zoom_distance);
        (*camera_layers).clone().without(1).with(2)
    };

    log_if_slow("toggle_map", start);
}

fn zoom(
    camera_offset: &mut CameraOffset,
    camera_layers: &mut Single<&mut RenderLayers, With<Camera3d>>,
    direction: ZoomDirection,
) {
    camera_offset.zoom(direction);

    if showing_map(camera_layers) {
        if camera_offset.zoom_tiles_only() {
            ***camera_layers = camera_layers.clone().with(1).without(2);
        }
    } else if camera_offset.zoom_map_only() {
        ***camera_layers = camera_layers.clone().without(1).with(2);
    }
}

fn showing_map(camera_layers: &RenderLayers) -> bool {
    camera_layers.intersects(&RenderLayers::layer(2))
}

fn reset_camera_angle(mut camera_offset: ResMut<CameraOffset>) {
    let start = Instant::now();

    camera_offset.reset_angle();

    log_if_slow("reset_camera_angle", start);
}

fn toggle_elevation(
    mut elevation_visibility: ResMut<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
) {
    let start = Instant::now();

    *elevation_visibility = match *elevation_visibility {
        ElevationVisibility::Shown => ElevationVisibility::Hidden,
        ElevationVisibility::Hidden => ElevationVisibility::Shown,
    };
    *visualization_update = VisualizationUpdate::Forced;

    log_if_slow("toggle_elevation", start);
}

pub(super) fn manage_mouse_scroll_input(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_offset: ResMut<CameraOffset>,
    mut camera_layers: Single<&mut RenderLayers, With<Camera3d>>,
    scroll_lists: Query<&Interaction, With<ScrollList>>,
) {
    let start = Instant::now();

    if scroll_lists.iter().all(|i| i == &Interaction::None) {
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
) {
    //trace!("{focus_state:?} {instruction:?}");
    match (*focus_state, &instruction) {
        (FocusState::Normal, _) => instruction_queue.add(instruction),
        (FocusState::ExaminingPos(target), QueuedInstruction::ToggleAutoTravel) => {
            //trace!("Autotravel pos");
            next_focus_state.set(FocusState::Normal);
            next_player_action_state.set(PlayerActionState::AutoTravel { target });
            instruction_queue.stop_waiting();
            message_writer.you("start traveling...").send_info();
        }
        (FocusState::ExaminingZoneLevel(zone_level), QueuedInstruction::ToggleAutoTravel) => {
            //trace!("Autotravel zone level");
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
            warn!("Ignoring {:?} in {:?}", &instruction, &focus_state);
        }
    }

    instruction_queue.log_if_long();
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_base_key_bindings(
    world: &mut World,
    held_key_bindings: Local<KeyBindings<GameplayScreenState, (), Held>>,
    fresh_key_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    held_key_bindings.spawn(world, GameplayScreenState::Base, |builder| {
        for &player_direction in PlayerDirection::VARIANTS {
            builder.add(
                player_direction.to_nbor(),
                (move || QueuedInstruction::Offset(player_direction))
                    .pipe(manage_queued_instruction),
            );
        }
    });

    world.spawn((
        ManualSection::new(&[("move", "numpad"), ("move up/down", "</>")], 100),
        StateScoped(GameplayScreenState::Base),
    ));

    fresh_key_bindings.spawn(world, GameplayScreenState::Base, |builder| {
        builder.add('m', (|| ZoomDistance::Close).pipe(toggle_map));
        builder.add('M', (|| ZoomDistance::Far).pipe(toggle_map));
        builder.add('x', examine_pos);
        builder.add('X', examine_zone_level);
        builder.add('&', open_crafting_screen);
        builder.add('i', open_inventory);
        builder.add('z', (|| ZoomDirection::In).pipe(manage_zoom));
        builder.add('Z', (|| ZoomDirection::Out).pipe(manage_zoom));
        builder.add('h', toggle_elevation);
        builder.add('0', reset_camera_angle);

        {
            use QueuedInstruction::{
                Attack, Close, Drag, Peek, Pulp, Sleep, Smash, ToggleAutoDefend, ToggleAutoTravel,
                Wait,
            };
            builder.add('|', (|| Wait).pipe(manage_queued_instruction));
            builder.add('$', (|| Sleep).pipe(manage_queued_instruction));
            builder.add('a', (|| Attack).pipe(manage_queued_instruction));
            builder.add('s', (|| Smash).pipe(manage_queued_instruction));
            builder.add('p', (|| Pulp).pipe(manage_queued_instruction));
            builder.add('c', (|| Close).pipe(manage_queued_instruction));
            builder.add('\\', (|| Drag).pipe(manage_queued_instruction));
            builder.add('G', (|| ToggleAutoTravel).pipe(manage_queued_instruction));
            builder.add('A', (|| ToggleAutoDefend).pipe(manage_queued_instruction));
            builder.add(KeyCode::Tab, (|| Peek).pipe(manage_queued_instruction));
        }

        builder.add(
            '+',
            (|| QueuedInstruction::ChangePace(ChangePace::Next)).pipe(manage_queued_instruction),
        );
        builder.add(
            '-',
            (|| QueuedInstruction::ChangePace(ChangePace::Previous))
                .pipe(manage_queued_instruction),
        );

        builder.add(KeyCode::Escape, handle_cancelation);
    });

    world.spawn((
        ManualSection::new(
            &[
                ("attack npc", "a"),
                ("smash furniture", "s"),
                ("pulp corpse", "p"),
                ("walking mode", "+/-"),
                ("auto defend", "A"),
                ("peek", "tab"),
                ("wait", "|"),
                ("sleep", "$"),
                ("show elevated", "h"),
                ("examine", "x"),
                ("examine map", "X"),
                ("auto travel", "G"),
                ("inventory", "i"),
                ("crafting", "&"),
                ("toggle map", "m/M"),
                ("camera angle", "middle mouse button"),
                ("reset angle", "0"),
                ("zoom", "z/Z"),
                ("zoom", "scroll wheel"),
            ],
            101,
        ),
        StateScoped(GameplayScreenState::Base),
    ));

    log_if_slow("create_crafting_key_bindings", start);
}

#[expect(clippy::needless_pass_by_value)]
fn handle_cancelation(
    mut message_writer: MessageWriter,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    player_action_state: Res<State<PlayerActionState>>,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    focus_state: Res<State<FocusState>>,
    mut next_focus_state: ResMut<NextState<FocusState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
) {
    let start = Instant::now();

    if focus_state.cancel_handling(&player_action_state) == CancelHandling::Menu {
        next_gameplay_state.set(GameplayScreenState::Menu);
    } else {
        handle_queued_instruction(
            &mut message_writer,
            &focus_state,
            &mut next_focus_state,
            &mut next_player_action_state,
            &mut instruction_queue,
            QueuedInstruction::CancelAction,
        );
    }

    log_if_slow("handle_cancelation", start);
}

fn manage_zoom(
    In(zoom_direction): In<ZoomDirection>,
    mut camera_offset: ResMut<CameraOffset>,
    mut camera_layers: Single<&mut RenderLayers, With<Camera3d>>,
) {
    let start = Instant::now();

    zoom(&mut camera_offset, &mut camera_layers, zoom_direction);

    log_if_slow("manage_zoom", start);
}

#[expect(clippy::needless_pass_by_value)]
fn manage_queued_instruction(
    In(instruction): In<QueuedInstruction>,
    mut message_writer: MessageWriter,
    focus_state: Res<State<FocusState>>,
    mut next_focus_state: ResMut<NextState<FocusState>>,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
) {
    let start = Instant::now();

    debug!("Player instruction: {:?}", &instruction);
    handle_queued_instruction(
        &mut message_writer,
        &focus_state,
        &mut next_focus_state,
        &mut next_player_action_state,
        &mut instruction_queue,
        instruction,
    );

    log_if_slow("manage_queued_instruction", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn update_camera_offset(
    camera_offset: Res<CameraOffset>,
    mut camera_transform: Single<&mut Transform, With<Camera3d>>,
) {
    let start = Instant::now();

    camera_transform.translation = camera_offset.offset();
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);

    log_if_slow("update_camera", start);
}

pub(in super::super) fn trigger_refresh(mut visualization_update: ResMut<VisualizationUpdate>) {
    *visualization_update = VisualizationUpdate::Forced;
}
