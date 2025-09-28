use crate::{
    ActorEvent, CorpseEvent, Damage, DespawnSubzoneLevel, DespawnZoneLevel, Healing, LogMessage,
    PlayerActionState, RefreshAfterBehavior, SpawnSubzoneLevel, SpawnZoneLevel, TerrainEvent,
    Toggle, UpdateZoneLevelVisibility,
};
use application_state::ApplicationState;
use bevy::prelude::{App, Plugin, StateScopedMessagesAppExt as _};

/// This plugin initializes all gameplay events and clears them between gameplays
pub(crate) struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ActorEvent<Damage>>()
            .clear_messages_on_exit::<ActorEvent<Damage>>(ApplicationState::Gameplay);
        app.add_message::<ActorEvent<Healing>>()
            .clear_messages_on_exit::<ActorEvent<Healing>>(ApplicationState::Gameplay);
        app.add_message::<CorpseEvent<Damage>>()
            .clear_messages_on_exit::<CorpseEvent<Damage>>(ApplicationState::Gameplay);
        app.add_message::<DespawnSubzoneLevel>()
            .clear_messages_on_exit::<DespawnSubzoneLevel>(ApplicationState::Gameplay);
        app.add_message::<DespawnZoneLevel>()
            .clear_messages_on_exit::<DespawnZoneLevel>(ApplicationState::Gameplay);
        app.add_message::<LogMessage>()
            .clear_messages_on_exit::<LogMessage>(ApplicationState::Gameplay);
        app.add_message::<LogMessage<PlayerActionState>>()
            .clear_messages_on_exit::<LogMessage<PlayerActionState>>(ApplicationState::Gameplay);
        app.add_message::<RefreshAfterBehavior>()
            .clear_messages_on_exit::<RefreshAfterBehavior>(ApplicationState::Gameplay);
        app.add_message::<SpawnSubzoneLevel>()
            .clear_messages_on_exit::<SpawnSubzoneLevel>(ApplicationState::Gameplay);
        app.add_message::<SpawnZoneLevel>()
            .clear_messages_on_exit::<SpawnZoneLevel>(ApplicationState::Gameplay);
        app.add_message::<TerrainEvent<Damage>>()
            .clear_messages_on_exit::<TerrainEvent<Damage>>(ApplicationState::Gameplay);
        app.add_message::<TerrainEvent<Toggle>>()
            .clear_messages_on_exit::<TerrainEvent<Toggle>>(ApplicationState::Gameplay);
        app.add_message::<UpdateZoneLevelVisibility>()
            .clear_messages_on_exit::<UpdateZoneLevelVisibility>(ApplicationState::Gameplay);
    }
}
