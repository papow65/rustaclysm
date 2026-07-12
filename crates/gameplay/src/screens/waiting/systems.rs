use crate::GameplayScreenState;
use crate::screens::waiting::{WaitDuration, YouWait};
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    AlignItems, Commands, DespawnOnExit, Entity, FlexDirection, In, IntoSystem as _,
    JustifyContent, KeyCode, Local, NextState, Node, ResMut, Val, World, info,
};
use gameplay_log::LogMessageWriter;
use gameplay_player::PlayerActionState;
use gameplay_time::Clock;
use hud::{
    BAD_TEXT_COLOR, ButtonBuilder, HARD_TEXT_COLOR, SMALL_SPACING, WARN_TEXT_COLOR,
    spawn_modal_panel, trigger_button_action,
};
use keyboard::KeyBindings;
use manual::ManualSection;
use std::time::Instant;
use strum::VariantArray as _;
use util::log_if_slow;

#[derive(Debug)]
pub(super) struct WaitingModalSystems {
    start_waiting_button: SystemId<In<WaitDuration>, ()>,
    start_waiting_key: SystemId<In<Entity>, ()>,
    cancel: SystemId<(), ()>,
}

pub(super) fn create_waiting_modal_system(world: &mut World) -> WaitingModalSystems {
    WaitingModalSystems {
        start_waiting_button: world.register_system_cached(start_waiting.pipe(exit_waiting_modal)),
        start_waiting_key: world.register_system_cached(trigger_button_action::<In<WaitDuration>>),
        cancel: world.register_system_cached(exit_waiting_modal),
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_wait_modal(
    In(waiting_modal_system): In<WaitingModalSystems>,
    mut commands: Commands,
    clock: Clock,
) {
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(30.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..Node::default()
    };

    let mut sorted_durations = WaitDuration::VARIANTS
        .iter()
        .map(|duration| (duration, duration.until(clock.time())))
        .collect::<Vec<_>>();
    sorted_durations.sort_by_key(|(_duration, until)| *until);

    let max_short_until = WaitDuration::ThirtyMinutes.until(clock.time());

    let modal_entity =
        spawn_modal_panel(&mut commands, GameplayScreenState::Waiting, Val::Px(320.0));

    commands.entity(modal_entity).with_children(|parent| {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: SMALL_SPACING,
                ..Node::default()
            })
            .with_children(|parent| {
                for (duration, until) in sorted_durations {
                    parent.spawn(
                        ButtonBuilder::new(
                            format!(
                                "{:<19} ({:02}:{:02})",
                                duration.message(),
                                (until.minute_of_day() / 60) % 24,
                                until.minute_of_day() % 60
                            ),
                            if until <= max_short_until {
                                HARD_TEXT_COLOR
                            } else {
                                WARN_TEXT_COLOR
                            },
                            waiting_modal_system.start_waiting_button,
                            duration.clone(),
                        )
                        .with_node(button_node.clone())
                        .key_binding(Some(duration.key()), waiting_modal_system.start_waiting_key)
                        .bundle(),
                    );
                }

                parent.spawn(
                    ButtonBuilder::new("Cancel", BAD_TEXT_COLOR, waiting_modal_system.cancel, ())
                        .with_node(button_node)
                        .bundle(),
                );
            });
    });
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_waiting_modal_key_bindings(
    world: &mut World,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    fresh_bindings.spawn(world, GameplayScreenState::Waiting, |bindings| {
        bindings.add(KeyCode::Escape, exit_waiting_modal);
        bindings.add('|', exit_waiting_modal);
    });

    world.spawn((
        ManualSection::new(&[("cancel", "esc/|")], 100),
        DespawnOnExit(GameplayScreenState::Waiting),
    ));

    log_if_slow("create_crafting_key_bindings", start);
}

#[expect(clippy::needless_pass_by_value)]
fn start_waiting(
    In(wait_timeout): In<WaitDuration>,
    mut message_writer: LogMessageWriter,
    mut next_state: ResMut<NextState<PlayerActionState>>,
    clock: Clock,
) {
    let start = Instant::now();

    let until = wait_timeout.until(clock.time());
    info!("start_waiting {until}");

    message_writer.send(YouWait);
    next_state.set(PlayerActionState::Waiting { until });

    log_if_slow("start_waiting", start);
}

fn exit_waiting_modal(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Base);

    log_if_slow("exit_waiting_modal", start);
}
