use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct MenuScreenPlugin;

impl Plugin for MenuScreenPlugin {
    fn build(&self, app: &mut App) {
        // startup
        app.add_systems((spawn_menu,).in_schedule(OnEnter(GameplayScreenState::Menu)));

        // every frame
        app.add_systems(
            (manage_menu_button_input, manage_menu_keyboard_input)
                .in_set(OnUpdate(ApplicationState::Gameplay))
                .in_set(OnUpdate(GameplayScreenState::Menu)),
        );

        // shutdown
        app.add_systems((despawn_menu,).in_schedule(OnExit(GameplayScreenState::Menu)));
    }
}
