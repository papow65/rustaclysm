use bevy::prelude::{Local, warn};
use std::time::{Duration, Instant};
use util::log_if_slow;

pub(super) struct StdInstant(Instant);

impl Default for StdInstant {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl StdInstant {
    pub(crate) fn next(&mut self) -> Duration {
        let previous = self.0;
        self.0 = Instant::now();
        self.0 - previous
    }
}

pub(super) fn check_delay(mut last_time: Local<StdInstant>) {
    let start = Instant::now();

    let delay = last_time.next();
    if Duration::from_millis(600) < delay {
        warn!("Very large delay: {delay:?}");
    }

    log_if_slow("check_delay", start);
}
