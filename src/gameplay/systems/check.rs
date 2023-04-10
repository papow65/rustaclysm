use crate::prelude::*;
use bevy::prelude::Local;
use std::time::{Duration, Instant};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn check_delay(mut last_time: Local<StdInstant>) {
    let start = Instant::now();

    let delay = last_time.next();
    if Duration::from_millis(600) < delay {
        eprintln!("Very large delay: {delay:?}");
    }

    log_if_slow("check_delay", start);
}
