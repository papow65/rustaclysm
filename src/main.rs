//! See <https://github.com/papow65/rustaclysm/blob/main/readme.md>

mod application;
mod background;
mod gameplay;
mod loading;
mod main_menu;
mod manual;
mod pre_gameplay;

use crate::application::run_application;
use bevy::prelude::AppExit;

fn main() -> AppExit {
    run_application()
}
