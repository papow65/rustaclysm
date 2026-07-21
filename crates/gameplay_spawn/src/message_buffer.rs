use bevy::ecs::system::SystemParam;
use bevy::prelude::{Local, Message, MessageReader};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(SystemParam)]
pub struct MessageBuffer<'w, 's, T: Message> {
    reader: MessageReader<'w, 's, T>,
    buffer: Local<'s, VecDeque<T>>,
}

impl<T: Clone + Copy + Message> MessageBuffer<'_, '_, T> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reader.len() == 0 && self.buffer.is_empty()
    }

    pub fn handle(&mut self, mut process: impl FnMut(T), max_duration: Duration) {
        if self.is_empty() {
            return;
        }

        //debug!(
        //    "MessageBuffer::handle: {} new + {} buffered messages",
        //    next.len(),
        //    self.reader.len()
        //);

        self.buffer.extend(self.reader.read());

        let now = Instant::now();
        while now.elapsed() <= max_duration
            && let Some(message) = self.buffer.pop_front()
        {
            process(message);
        }
        //debug!(
        //    "MessageBuffer::handle: {} buffered messages left",
        //    self.cache.len()
        //);
    }
}
