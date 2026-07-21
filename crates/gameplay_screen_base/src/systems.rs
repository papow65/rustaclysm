use crate::messages::{YouAreBusy, YouStartTraveling};
use bevy::prelude::{
    DespawnOnExit, In, IntoSystem as _, KeyCode, Local, NextState, Res, ResMut, State, World,
    debug, warn,
};
use gameplay_action_planning::{PlayerDirection, PlayerInstructions, QueuedInstruction};
use gameplay_camera::{ZoomDirection, ZoomDistance, manage_zoom, reset_camera_angle, toggle_map};
use gameplay_character::ChangePace;
use gameplay_focus::{CancelHandling, ElevationVisibility, Focus, FocusState};
use gameplay_log::LogMessageWriter;
use gameplay_player::PlayerActionState;
use gameplay_screen_state::GameplayScreenState;
use gameplay_visualization::VisualizationUpdate;
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

#[expect(clippy::needless_pass_by_value)]
fn open_screen(
    In(screen): In<GameplayScreenState>,
    player_action_state: Res<State<PlayerActionState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut message_writer: LogMessageWriter,
) {
    let start = Instant::now();

    if player_action_state.is_automatic() {
        message_writer.send(YouAreBusy);
    } else {
        next_gameplay_state.set(screen);
    }

    log_if_slow("open_screen", start);
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

fn handle_queued_instruction(
    message_writer: &mut LogMessageWriter,
    player_instructions: &mut PlayerInstructions,
    focus_state: &FocusState,
    next_focus_state: &mut ResMut<NextState<FocusState>>,
    next_player_action_state: &mut ResMut<NextState<PlayerActionState>>,
    instruction: QueuedInstruction,
) {
    //trace!("{focus_state:?} {instruction:?}");
    match (*focus_state, &instruction) {
        (FocusState::Normal, _) => player_instructions.push(instruction),
        (FocusState::ExaminingPos(target), QueuedInstruction::ToggleAutoTravel) => {
            //trace!("Autotravel pos");
            next_focus_state.set(FocusState::Normal);
            next_player_action_state.set(PlayerActionState::AutoTravel { target });
            message_writer.send(YouStartTraveling);
        }
        (FocusState::ExaminingZoneLevel(zone_level), QueuedInstruction::ToggleAutoTravel) => {
            //trace!("Autotravel zone level");
            next_focus_state.set(FocusState::Normal);
            next_player_action_state.set(PlayerActionState::AutoTravel {
                target: zone_level.center_pos(),
            });
            message_writer.send(YouStartTraveling);
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

    player_instructions.log_if_long();
}

#[expect(clippy::needless_pass_by_value)]
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
        DespawnOnExit(GameplayScreenState::Base),
    ));

    fresh_key_bindings.spawn(world, GameplayScreenState::Base, |builder| {
        builder.add('m', (|| ZoomDistance::Close).pipe(toggle_map));
        builder.add('M', (|| ZoomDistance::Far).pipe(toggle_map));
        builder.add('x', examine_pos);
        builder.add('X', examine_zone_level);
        builder.add('&', (|| GameplayScreenState::Crafting).pipe(open_screen));
        builder.add('i', (|| GameplayScreenState::Inventory).pipe(open_screen));
        builder.add('q', (|| GameplayScreenState::Quality).pipe(open_screen));
        builder.add('t', (|| GameplayScreenState::Tool).pipe(open_screen));
        builder.add('|', (|| GameplayScreenState::Waiting).pipe(open_screen));
        builder.add('z', (|| ZoomDirection::In).pipe(manage_zoom));
        builder.add('Z', (|| ZoomDirection::Out).pipe(manage_zoom));
        builder.add('h', toggle_elevation);
        builder.add('0', reset_camera_angle);

        {
            use QueuedInstruction::{
                Attack, Close, Drag, Peek, Pulp, Sleep, Smash, ToggleAutoDefend, ToggleAutoTravel,
            };
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
                ("qualities", "q"),
                ("tool actions", "t"),
                ("toggle map", "m/M"),
                ("camera angle", "middle mouse button"),
                ("reset angle", "0"),
                ("zoom", "z/Z"),
                ("zoom", "scroll wheel"),
            ],
            101,
        ),
        DespawnOnExit(GameplayScreenState::Base),
    ));

    log_if_slow("create_crafting_key_bindings", start);
}

#[expect(clippy::needless_pass_by_value)]
fn handle_cancelation(
    mut message_writer: LogMessageWriter,
    mut player_instructions: ResMut<PlayerInstructions>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    player_action_state: Res<State<PlayerActionState>>,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
    focus_state: Res<State<FocusState>>,
    mut next_focus_state: ResMut<NextState<FocusState>>,
) {
    let start = Instant::now();

    if focus_state.cancel_handling(&player_action_state) == CancelHandling::Menu {
        next_gameplay_state.set(GameplayScreenState::Menu);
    } else {
        handle_queued_instruction(
            &mut message_writer,
            &mut player_instructions,
            &focus_state,
            &mut next_focus_state,
            &mut next_player_action_state,
            // &player_instructions,
            QueuedInstruction::CancelAction,
        );
    }

    log_if_slow("handle_cancelation", start);
}

#[expect(clippy::needless_pass_by_value)]
fn manage_queued_instruction(
    In(instruction): In<QueuedInstruction>,
    mut message_writer: LogMessageWriter,
    mut player_instructions: ResMut<PlayerInstructions>,
    focus_state: Res<State<FocusState>>,
    mut next_focus_state: ResMut<NextState<FocusState>>,
    mut next_player_action_state: ResMut<NextState<PlayerActionState>>,
) {
    let start = Instant::now();

    debug!("Player instruction: {:?}", &instruction);
    handle_queued_instruction(
        &mut message_writer,
        &mut player_instructions,
        &focus_state,
        &mut next_focus_state,
        &mut next_player_action_state,
        instruction,
    );

    log_if_slow("manage_queued_instruction", start);
}

pub(crate) fn trigger_refresh(mut visualization_update: ResMut<VisualizationUpdate>) {
    *visualization_update = VisualizationUpdate::Forced;
}
