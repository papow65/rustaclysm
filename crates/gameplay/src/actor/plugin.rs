use crate::{ActorEvent, BehaviorPlugin, CorpseEvent};
use application_state::ApplicationState;
use bevy::prelude::{App, AppExtStates as _, Plugin, StateScopedMessagesAppExt as _};
use gameplay_common::{Damage, Healing};
use gameplay_player::PlayerActionState;
use util::log_transition_plugin;

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<PlayerActionState>();
        app.add_plugins((BehaviorPlugin, log_transition_plugin::<PlayerActionState>));

        app.add_message::<ActorEvent<Damage>>()
            .clear_messages_on_exit::<ActorEvent<Damage>>(ApplicationState::Gameplay);
        app.add_message::<ActorEvent<Healing>>()
            .clear_messages_on_exit::<ActorEvent<Healing>>(ApplicationState::Gameplay);
        app.add_message::<CorpseEvent<Damage>>()
            .clear_messages_on_exit::<CorpseEvent<Damage>>(ApplicationState::Gameplay);
    }
}
