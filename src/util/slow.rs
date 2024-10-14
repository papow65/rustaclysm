use std::time::{Duration, Instant};

const MAX_SYSTEM_DURATION: Duration = Duration::from_micros(300);

pub(crate) fn log_if_slow(name: &str, start: Instant) {
    if cfg!(debug_assertions) {
        let duration = start.elapsed();
        if 5 * MAX_SYSTEM_DURATION < duration {
            eprintln!("Very slow system: {name} took {duration:?}");
        } else if MAX_SYSTEM_DURATION < duration {
            println!("Slow system: {name} took {duration:?}");
        }
    }
}
