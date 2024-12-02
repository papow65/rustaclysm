//! See <https://github.com/papow65/rustaclysm/blob/main/readme.md>

mod application;
mod background;
mod gameplay;
mod hud;
mod keyboard;
mod loading;
mod main_menu;
mod manual;
mod pre_gameplay;
mod util;

use crate::application::run_application;
use bevy::prelude::AppExit;
use std::env;

fn main() -> AppExit {
    const RUST_BACKTRACE: &str = "RUST_BACKTRACE";
    if env::var_os(RUST_BACKTRACE).is_none() {
        env::set_var(RUST_BACKTRACE, "1");
    }

    run_application()
}
