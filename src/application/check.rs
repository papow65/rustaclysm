use crate::prelude::*;
use bevy::prelude::Local;
use std::time::{Duration, Instant};

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

#[allow(clippy::needless_pass_by_value)]
pub(super) fn check_delay(mut last_time: Local<StdInstant>) {
    let start = Instant::now();

    let delay = last_time.next();
    if Duration::from_millis(600) < delay {
        eprintln!("Very large delay: {delay:?}");
    }

    log_if_slow("check_delay", start);
}
