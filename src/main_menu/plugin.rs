use crate::application::ApplicationState;
use crate::main_menu::systems::{
    FoundSav, create_load_systems, create_main_menu_key_bindings, create_quit_system,
    enter_main_menu, spawn_main_menu, update_sav_files,
};
use bevy::prelude::{
    App, FixedUpdate, In, IntoSystem as _, IntoSystemConfigs as _, OnEnter, Plugin, Startup,
    Update, in_state,
};
use hud::manage_button_input;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, enter_main_menu);

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
