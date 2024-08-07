use crate::application::ApplicationState;
use crate::main_menu::systems::{manage_main_menu_button_input, spawn_main_menu, update_sav_files};
use bevy::prelude::{in_state, App, FixedUpdate, IntoSystemConfigs, OnEnter, Plugin, Update};

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::MainMenu),
            (spawn_main_menu, update_sav_files).chain(),
        );

        app.add_systems(
            Update,
            manage_main_menu_button_input.run_if(in_state(ApplicationState::MainMenu)),
        );

        app.add_systems(
            FixedUpdate,
            update_sav_files.run_if(in_state(ApplicationState::MainMenu)),
        );
    }
}
