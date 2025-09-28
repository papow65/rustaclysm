use crate::{Intransient, LogMessage, LogMessageTransience, PlayerActionState, ProtoPhrase};
use bevy::{ecs::system::SystemParam, prelude::MessageWriter};

#[derive(SystemParam)]
pub(crate) struct LogMessageWriter<'w, T: LogMessageTransience = Intransient> {
    event_writer: MessageWriter<'w, LogMessage<T>>,
}

impl LogMessageWriter<'_, Intransient> {
    pub(crate) fn send<P: ProtoPhrase>(&mut self, phrase: P) {
        self.event_writer
            .write(LogMessage::new(phrase.compose(), P::SEVERITY));
    }
}

impl LogMessageWriter<'_, PlayerActionState> {
    pub(crate) fn send_transient<P: ProtoPhrase>(&mut self, phrase: P, state: PlayerActionState) {
        self.event_writer.write(LogMessage::new_transient(
            phrase.compose(),
            P::SEVERITY,
            state,
        ));
    }
}
