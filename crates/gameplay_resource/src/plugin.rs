use crate::{GampelayResourceSet, add_resource, remove_resource};
use application_state::ApplicationState;
use bevy::prelude::{App, IntoScheduleConfigs as _, OnEnter, OnExit, Resource};

/// This creates the resource when gameplay starts, and removes it when gameplay ends.
pub fn gameplay_resource_plugin<R: Default + Resource>(app: &mut App) {
    app.add_systems(
        OnEnter(ApplicationState::Gameplay),
        add_resource::<R>.in_set(GampelayResourceSet),
    );
    app.add_systems(OnExit(ApplicationState::Gameplay), remove_resource::<R>);
}
