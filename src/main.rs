//! See <https://github.com/papow65/rustaclysm/blob/main/readme.md>

mod application;
mod background;
mod gameplay;
mod hud;
mod keyboard;
mod loading;
mod main_menu;
mod manual;
mod util;

use crate::application::run_application;
use bevy::prelude::AppExit;
use std::env;

fn main() -> AppExit {
    let rust_backtrace = "RUST_BACKTRACE";
    if env::var_os(rust_backtrace).is_none() {
        env::set_var(rust_backtrace, "1");
    }

    run_application()
}
