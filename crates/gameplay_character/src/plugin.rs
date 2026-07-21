use crate::{CharacterEvent, CorpseEvent};
use application_state::ApplicationState;
use bevy::prelude::{App, Plugin, StateScopedMessagesAppExt as _};
use gameplay_object::{Damage, Healing};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CharacterEvent<Damage>>()
            .clear_messages_on_exit::<CharacterEvent<Damage>>(ApplicationState::Gameplay);
        app.add_message::<CharacterEvent<Healing>>()
            .clear_messages_on_exit::<CharacterEvent<Healing>>(ApplicationState::Gameplay);
        app.add_message::<CorpseEvent<Damage>>()
            .clear_messages_on_exit::<CorpseEvent<Damage>>(ApplicationState::Gameplay);
    }
}
