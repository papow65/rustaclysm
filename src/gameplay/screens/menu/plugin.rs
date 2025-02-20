use crate::gameplay::GameplayScreenState;
use crate::gameplay::screens::menu::systems::{
    create_menu_button_actions, create_menu_key_bindings, spawn_menu,
};
use bevy::prelude::{App, IntoSystem as _, OnEnter, Plugin};

pub(crate) struct MenuScreenPlugin;

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
