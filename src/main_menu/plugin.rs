use crate::application::ApplicationState;
use crate::hud::manage_button_input;
use crate::main_menu::systems::{
    create_load_systems, create_main_menu_key_bindings, create_quit_system, spawn_main_menu,
    update_sav_files, FoundSav,
};
use bevy::prelude::{
    in_state, App, FixedUpdate, In, IntoSystem, IntoSystemConfigs, OnEnter, Plugin, Update,
};

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::MainMenu),
            (
                create_quit_system.pipe(spawn_main_menu),
                create_main_menu_key_bindings,
            ),
        );

        app.add_systems(
            Update,
            manage_button_input::<In<FoundSav>>.run_if(in_state(ApplicationState::MainMenu)),
        );

        app.add_systems(
            FixedUpdate,
            create_load_systems
                .pipe(update_sav_files)
                .run_if(in_state(ApplicationState::MainMenu)),
        );
    }
}
