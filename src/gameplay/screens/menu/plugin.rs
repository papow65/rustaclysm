use crate::gameplay::screens::menu::systems::{
    create_menu_key_bindings, manage_menu_button_input, spawn_menu,
};
use crate::gameplay::GameplayScreenState;
use bevy::prelude::{in_state, App, IntoSystemConfigs, OnEnter, Plugin, Update};

pub(crate) struct MenuScreenPlugin;

impl Plugin for MenuScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Menu),
            (spawn_menu, create_menu_key_bindings),
        );

        app.add_systems(
            Update,
            manage_menu_button_input.run_if(in_state(GameplayScreenState::Menu)),
        );
    }
}
