use bevy::prelude::{
    App, IntoScheduleConfigs as _, MessageReader, Res, Resource, StateTransitionEvent, States,
    Update, info, resource_exists_and_changed,
};
use std::{any::type_name, fmt::Debug};

pub fn log_transition_plugin<S: States>(app: &mut App) {
    if cfg!(debug_assertions) {
        app.add_systems(Update, log_transitions::<S>);
    }
}

/// Logs state transitions into console.
///
/// This system is provided to make debugging easier by tracking state changes.
///
/// Based on <https://github.com/bevyengine/bevy/blob/release-0.20.0/crates/bevy_dev_tools/src/states.rs>
fn log_transitions<S: States>(mut transitions: MessageReader<StateTransitionEvent<S>>) {
    // State internals can generate at most one event (of type) per frame.
    let Some(transition) = transitions.read().last() else {
        return;
    };
    let name = core::any::type_name::<S>();
    let StateTransitionEvent {
        exited,
        entered,
        allow_same_state_transitions,
    } = transition;
    let skip_text = if exited == entered && !*allow_same_state_transitions {
        " (disallowing same-state transitions)"
    } else {
        ""
    };
    info!("{name} transition: {exited:?} => {entered:?}{skip_text}");
}

pub fn log_resource_change_plugin<R: Resource + Debug>(app: &mut App) {
    if cfg!(debug_assertions) {
        app.add_systems(
            Update,
            log_resource_change::<R>.run_if(resource_exists_and_changed::<R>),
        );
    }
}

// Inspired by bevy's log_transitions
#[expect(clippy::needless_pass_by_value)]
fn log_resource_change<R: Resource + Debug>(resource: Res<R>) {
    let name = type_name::<R>();
    let resource = &*resource;
    info!("{name} changed to {resource:?}");
}
