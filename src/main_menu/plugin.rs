use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        // startup
        app.add_systems(
            OnEnter(ApplicationState::MainMenu),
            (
                spawn_main_menu,
                apply_deferred,
                resize_background,
                update_sav_files,
            )
                .chain(),
        );

        // every frame
        app.add_systems(
            Update,
            (
                manage_main_menu_button_input,
                manage_main_menu_keyboard_input,
            )
                .run_if(in_state(ApplicationState::MainMenu)),
        );

        // frequent, but not every frame
        app.add_systems(
            FixedUpdate,
            (update_sav_files, resize_background).run_if(in_state(ApplicationState::MainMenu)),
        );

        // shutdown
        app.add_systems(OnExit(ApplicationState::MainMenu), despawn_main_menu);
    }
}
