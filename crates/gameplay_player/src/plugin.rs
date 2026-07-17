use crate::PlayerActionState;
use application_state::ApplicationState;
use bevy::prelude::{App, Plugin, StateScopedMessagesAppExt as _};
use gameplay_log::LogMessage;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LogMessage<PlayerActionState>>()
            .clear_messages_on_exit::<LogMessage<PlayerActionState>>(ApplicationState::Gameplay);
    }
}
