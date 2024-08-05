use crate::application::ApplicationState;
use crate::main_menu::systems::{
    manage_main_menu_button_input, resize_background, spawn_main_menu, update_sav_files,
};
use bevy::{
    prelude::{in_state, on_event, App, FixedUpdate, IntoSystemConfigs, OnEnter, Plugin, Update},
    window::WindowResized,
};

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::MainMenu),
            (spawn_main_menu, resize_background, update_sav_files).chain(),
        );

        app.add_systems(
            Update,
            (
                manage_main_menu_button_input,
                resize_background.run_if(on_event::<WindowResized>()),
            )
                .run_if(in_state(ApplicationState::MainMenu)),
        );

        app.add_systems(
            FixedUpdate,
            (update_sav_files, resize_background).run_if(in_state(ApplicationState::MainMenu)),
        );
    }
}
