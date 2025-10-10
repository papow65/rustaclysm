use bevy::ecs::system::SystemParam;
use bevy::prelude::{Local, Message, MessageReader};
use std::time::{Duration, Instant};

#[derive(SystemParam)]
pub struct MessageBuffer<'w, 's, T: Message> {
    reader: MessageReader<'w, 's, T>,
    cache: Local<'s, Vec<T>>,
}

impl<T: Clone + Copy + Message> MessageBuffer<'_, '_, T> {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reader.len() == 0 && self.cache.is_empty()
    }

    pub fn handle(&mut self, mut process: impl FnMut(T), max_duration: Duration) {
        if self.is_empty() {
            return;
        }

        let mut next = self.reader.read();
        //debug!(
        //    "MessageBuffer::handle: {} new + {} cached messages",
        //    next.len(),
        //    self.cache.len()
        //);
        let now = Instant::now();
        while now.elapsed() <= max_duration
            && let Some(message) = next.next()
        {
            process(*message);
        }
        self.cache.extend(next.copied());

        while now.elapsed() <= max_duration
            && let Some(message) = self.cache.pop()
        {
            process(message);
        }
        //debug!(
        //    "MessageBuffer::handle: {} cached messages left",
        //    self.cache.len()
        //);
    }
}
