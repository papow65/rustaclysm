pub mod character;
pub mod check;
pub mod hud;
pub mod input;
pub mod spawn;
pub mod startup;
pub mod update;

use std::time::{Duration, Instant};

fn log_if_slow(name: &str, start: Instant) {
    let duration = Instant::now() - start;
    if Duration::new(0, 200_000) < duration {
        println!("slow system: {name} took {duration:?}");
    }
}
