use crate::{
    DespawnSubzoneLevel, DespawnZoneLevel, SpawnSubzoneLevel, SpawnZoneLevel,
    UpdateZoneLevelVisibility,
};
use application_state::ApplicationState;
use bevy::prelude::{App, Plugin, StateScopedMessagesAppExt as _};

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DespawnSubzoneLevel>()
            .clear_messages_on_exit::<DespawnSubzoneLevel>(ApplicationState::Gameplay);
        app.add_message::<DespawnZoneLevel>()
            .clear_messages_on_exit::<DespawnZoneLevel>(ApplicationState::Gameplay);
        app.add_message::<SpawnSubzoneLevel>()
            .clear_messages_on_exit::<SpawnSubzoneLevel>(ApplicationState::Gameplay);
        app.add_message::<SpawnZoneLevel>()
            .clear_messages_on_exit::<SpawnZoneLevel>(ApplicationState::Gameplay);
        app.add_message::<UpdateZoneLevelVisibility>()
            .clear_messages_on_exit::<UpdateZoneLevelVisibility>(ApplicationState::Gameplay);
    }
}
