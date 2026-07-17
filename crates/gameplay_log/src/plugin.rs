use crate::LogMessage;
use application_state::ApplicationState;
use bevy::prelude::{App, Plugin, StateScopedMessagesAppExt as _};

pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LogMessage>()
            .clear_messages_on_exit::<LogMessage>(ApplicationState::Gameplay);
    }
}
