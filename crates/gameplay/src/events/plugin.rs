use crate::{
    ActorEvent, CorpseEvent, Damage, DespawnSubzoneLevel, DespawnZoneLevel, Healing, Message,
    PlayerActionState, RefreshAfterBehavior, SpawnSubzoneLevel, SpawnZoneLevel, TerrainEvent,
    Toggle, UpdateZoneLevelVisibility,
};
use application_state::ApplicationState;
use bevy::prelude::{App, Plugin, StateScopedEventsAppExt as _};

/// This plugin initializes all gameplay events and clears them between gameplays
pub(crate) struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_state_scoped_event::<ActorEvent<Damage>>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<ActorEvent<Healing>>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<CorpseEvent<Damage>>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<DespawnSubzoneLevel>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<DespawnZoneLevel>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<Message>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<Message<PlayerActionState>>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<RefreshAfterBehavior>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<SpawnSubzoneLevel>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<SpawnZoneLevel>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<TerrainEvent<Damage>>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<TerrainEvent<Toggle>>(ApplicationState::Gameplay);
        app.add_state_scoped_event::<UpdateZoneLevelVisibility>(ApplicationState::Gameplay);
    }
}
