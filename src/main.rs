//! See <https://github.com/papow65/rustaclysm/blob/main/readme.md>

mod application;
mod background;
mod common;
mod gameplay;
mod hud;
mod keyboard;
mod loading;
mod main_menu;
mod manual;

use crate::application::run_application;
use std::env;

fn main() {
    let rust_backtrace = "RUST_set_varBACKTRACE";
    if env::var_os(rust_backtrace).is_none() {
        env::set_var(rust_backtrace, "1");
    }

    run_application();
}
