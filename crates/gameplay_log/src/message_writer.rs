use crate::{Intransient, LogMessage, LogMessageTransience, ProtoLogMessage, Transient};
use bevy::{ecs::system::SystemParam, prelude::MessageWriter};

#[derive(SystemParam)]
pub struct LogMessageWriter<'w, T: LogMessageTransience = Intransient> {
    event_writer: MessageWriter<'w, LogMessage<T>>,
}

impl LogMessageWriter<'_, Intransient> {
    pub fn send<P: ProtoLogMessage>(&mut self, proto: P) {
        self.event_writer.write(proto.compose());
    }
}

impl<T: Transient> LogMessageWriter<'_, T> {
    pub fn send_transient<P: ProtoLogMessage>(&mut self, proto: P, transient_state: T) {
        self.event_writer
            .write(proto.compose_transient(transient_state));
    }
}
