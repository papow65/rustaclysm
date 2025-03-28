use crate::application::ApplicationState;
use crate::gameplay::scope::gameplay_counter::GameplayCounter;
use crate::gameplay::spawn::spawn_initial_entities;
use bevy::prelude::{
    App, Commands, IntoScheduleConfigs as _, OnEnter, OnExit, Plugin, ResMut, Resource,
};
use std::marker::PhantomData;

/// This makes make [`GameplayLocal`](`crate::gameplay::scope::GameplayLocal`) work, by creating and increasing a [`GameplayCounter`].
pub(crate) struct GameplayLocalPlugin;

impl Plugin for GameplayLocalPlugin {
    fn build(&self, app: &mut App) {
        // GameplayCounter is kept between gameplay sessions
        app.insert_resource(GameplayCounter::default());

        app.add_systems(OnExit(ApplicationState::Gameplay), increase_counter);
    }
}

fn increase_counter(mut counter: ResMut<GameplayCounter>) {
    counter.increase();
}

/// This creates the resource when gameplay starts, and removes it when gameplay ends.
pub(crate) struct GameplayResourcePlugin<T: Resource>(PhantomData<T>);

impl<T: Default + Resource> Default for GameplayResourcePlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Default + Resource> Plugin for GameplayResourcePlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            add_resource::<T>.before(spawn_initial_entities),
        );
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_resource::<T>);
    }
}

fn add_resource<T: Default + Resource>(mut commands: Commands) {
    commands.insert_resource(T::default());
}

fn remove_resource<T: Resource>(mut commands: Commands) {
    commands.remove_resource::<T>();
}
