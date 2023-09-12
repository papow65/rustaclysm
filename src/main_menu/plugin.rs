use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
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

        app.add_systems(
            Update,
            manage_main_menu_button_input.run_if(in_state(ApplicationState::MainMenu)),
        );

        app.add_systems(
            FixedUpdate,
            (update_sav_files, resize_background).run_if(in_state(ApplicationState::MainMenu)),
        );

        app.add_systems(
            OnExit(ApplicationState::MainMenu),
            despawn::<ApplicationState>,
        );
    }
}
