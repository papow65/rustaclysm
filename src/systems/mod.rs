mod character;
mod check;
mod hud;
mod input;
mod spawn;
mod startup;
mod update;

use std::time::{Duration, Instant};

pub(crate) use self::{character::*, check::*, hud::*, input::*, spawn::*, startup::*, update::*};

pub(crate) const MAX_SYSTEM_DURATION: Duration = Duration::from_micros(300);

pub(crate) fn log_if_slow(name: &str, start: Instant) {
    let duration = start.elapsed();
    if 5 * MAX_SYSTEM_DURATION < duration {
        eprintln!("Very slow system: {name} took {duration:?}");
    } else if MAX_SYSTEM_DURATION < duration {
        println!("Slow system: {name} took {duration:?}");
    }
}
