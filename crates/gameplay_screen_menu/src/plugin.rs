use crate::systems::{create_menu_button_actions, create_menu_key_bindings, spawn_menu};
use bevy::prelude::{App, IntoSystem as _, OnEnter, Plugin};
use gameplay_screen_state::GameplayScreenState;

pub struct MenuScreenPlugin;

impl Plugin for MenuScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Menu),
            (
                create_menu_button_actions.pipe(spawn_menu),
                create_menu_key_bindings,
            ),
        );
    }
}
