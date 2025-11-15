use bevy::dev_tools::states::log_transitions;
use bevy::prelude::{
    App, IntoScheduleConfigs as _, Res, Resource, States, Update, info, resource_exists_and_changed,
};
use std::{any::type_name, fmt::Debug};

pub fn log_transition_plugin<S: States>(app: &mut App) {
    if cfg!(debug_assertions) {
        app.add_systems(Update, log_transitions::<S>);
    }
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
