mod components;
mod load_error;
mod plugin;
mod systems;

pub use crate::plugin::MainMenuPlugin;

use crate::components::{LoadButtonArea, MessageField, MessageWrapper};
use crate::load_error::LoadError;
use crate::systems::{
    FoundSav, create_load_systems, create_main_menu_key_bindings, create_quit_system,
    enter_main_menu, spawn_main_menu, update_sav_files,
};
