use crate::{Intransient, LogMessage, LogMessageTransience, PlayerActionState, ProtoLogMessage};
use bevy::{ecs::system::SystemParam, prelude::MessageWriter};

#[derive(SystemParam)]
pub(crate) struct LogMessageWriter<'w, T: LogMessageTransience = Intransient> {
    event_writer: MessageWriter<'w, LogMessage<T>>,
}

impl LogMessageWriter<'_, Intransient> {
    pub(crate) fn send<P: ProtoLogMessage>(&mut self, proto: P) {
        self.event_writer.write(proto.compose());
    }
}

impl LogMessageWriter<'_, PlayerActionState> {
    pub(crate) fn send_transient<P: ProtoLogMessage>(
        &mut self,
        proto: P,
        state: PlayerActionState,
    ) {
        self.event_writer.write(proto.compose_transient(state));
    }
}
