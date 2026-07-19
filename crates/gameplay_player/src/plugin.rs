use crate::PlayerActionState;
use application_state::ApplicationState;
use bevy::prelude::{App, AppExtStates as _, Plugin, StateScopedMessagesAppExt as _};
use gameplay_log::LogMessage;
use util::log_transition_plugin;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<PlayerActionState>();
        app.add_plugins(log_transition_plugin::<PlayerActionState>);

        app.add_message::<LogMessage<PlayerActionState>>()
            .clear_messages_on_exit::<LogMessage<PlayerActionState>>(ApplicationState::Gameplay);
    }
}
