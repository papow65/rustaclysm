use std::marker::PhantomData;

use crate::application::ApplicationState;
use crate::gameplay::{
    ActorEvent, CorpseEvent, Damage, DespawnSubzoneLevel, DespawnZoneLevel, Healing, Message,
    RefreshAfterBehavior, SpawnSubzoneLevel, SpawnZoneLevel, TerrainEvent, Toggle,
    UpdateZoneLevelVisibility,
};
use bevy::prelude::{App, Event, Events, OnExit, Plugin, ResMut};

/// This plugin initializes all gameplay events and clears them between gameplays
pub(crate) struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EventPlugin::<ActorEvent<Damage>>::default(),
            EventPlugin::<ActorEvent<Healing>>::default(),
            EventPlugin::<CorpseEvent<Damage>>::default(),
            EventPlugin::<DespawnSubzoneLevel>::default(),
            EventPlugin::<DespawnZoneLevel>::default(),
            EventPlugin::<Message>::default(),
            EventPlugin::<RefreshAfterBehavior>::default(),
            EventPlugin::<SpawnSubzoneLevel>::default(),
            EventPlugin::<SpawnZoneLevel>::default(),
            EventPlugin::<TerrainEvent<Damage>>::default(),
            EventPlugin::<TerrainEvent<Toggle>>::default(),
            EventPlugin::<UpdateZoneLevelVisibility>::default(),
        ));
    }
}

/// This plugin initializes a single gameplay event type and clears the event type between gameplays
pub(crate) struct EventPlugin<E: Event>(PhantomData<E>);

impl<E: Event> Default for EventPlugin<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E: Event> Plugin for EventPlugin<E> {
    fn build(&self, app: &mut App) {
        // app.add_events is better than creating event resources.
        // See https://github.com/TheBevyFlock/bevy_cli/blob/main/bevy_lint/src/lints/insert_event_resource.rs
        app.add_event::<E>();

        app.add_systems(OnExit(ApplicationState::Gameplay), clear_events::<E>);
    }
}

fn clear_events<T: Event>(mut events: ResMut<Events<T>>) {
    events.clear();
}
