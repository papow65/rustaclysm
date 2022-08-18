mod character;
mod check;
mod hud;
mod input;
mod spawn;
mod startup;
mod update;

use std::time::{Duration, Instant};

pub(crate) use self::{character::*, check::*, hud::*, input::*, spawn::*, startup::*, update::*};

fn log_if_slow(name: &str, start: Instant) {
    let duration = Instant::now() - start;
    if Duration::new(0, 200_000) < duration {
        println!("slow system: {name} took {duration:?}");
    }
}
