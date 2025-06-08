use crate::{GampelayResourceSet, add_resource, remove_resource};
use application_state::ApplicationState;
use bevy::prelude::{App, IntoScheduleConfigs as _, OnEnter, OnExit, Plugin, Resource};
use std::marker::PhantomData;

/// This creates the resource when gameplay starts, and removes it when gameplay ends.
pub struct GameplayResourcePlugin<T: Resource>(PhantomData<T>);

impl<T: Default + Resource> Default for GameplayResourcePlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Default + Resource> Plugin for GameplayResourcePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            add_resource::<T>.in_set(GampelayResourceSet),
        );
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_resource::<T>);
    }
}
