use crate::{Intransient, Message, MessageTransience, PlayerActionState, ProtoPhrase};
use bevy::{ecs::system::SystemParam, prelude::EventWriter};

#[derive(SystemParam)]
pub(crate) struct MessageWriter<'w, T: MessageTransience = Intransient> {
    event_writer: EventWriter<'w, Message<T>>,
}

impl MessageWriter<'_, Intransient> {
    pub(crate) fn send<P: ProtoPhrase>(&mut self, phrase: P) {
        self.event_writer
            .write(Message::new(phrase.compose(), P::SEVERITY));
    }
}

impl MessageWriter<'_, PlayerActionState> {
    pub(crate) fn send_transient<P: ProtoPhrase>(&mut self, phrase: P, state: PlayerActionState) {
        self.event_writer
            .write(Message::new_transient(phrase.compose(), P::SEVERITY, state));
    }
}
