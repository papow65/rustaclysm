use crate::{TerrainEvent, Toggle};
use application_state::ApplicationState;
use bevy::prelude::{App, Plugin, StateScopedMessagesAppExt as _};
use gameplay_object::Damage;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TerrainEvent<Damage>>()
            .clear_messages_on_exit::<TerrainEvent<Damage>>(ApplicationState::Gameplay);
        app.add_message::<TerrainEvent<Toggle>>()
            .clear_messages_on_exit::<TerrainEvent<Toggle>>(ApplicationState::Gameplay);
    }
}
