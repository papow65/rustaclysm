use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FixedTime::new_from_secs(0.25));

        // Main menu startup
        app.add_systems(
            (spawn_main_menu, apply_system_buffers, update_sav_files)
                .in_schedule(OnEnter(ApplicationState::MainMenu)),
        );

        // Every frame
        app.add_systems(
            (
                manage_main_menu_button_input,
                manage_main_menu_keyboard_input,
            )
                .in_set(OnUpdate(ApplicationState::MainMenu)),
        );

        app.add_system(
            update_sav_files
                .in_schedule(CoreSchedule::FixedUpdate)
                .in_set(OnUpdate(ApplicationState::MainMenu)),
        );

        // Main menu shutdown
        app.add_systems((despawn_main_menu,).in_schedule(OnExit(ApplicationState::MainMenu)));
    }
}
