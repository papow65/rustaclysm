use crate::gameplay::screens::menu::systems::{
    manage_menu_button_input, manage_menu_keyboard_input, spawn_menu,
};
use crate::prelude::GameplayScreenState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{in_state, on_event, App, IntoSystemConfigs, OnEnter, Plugin, Update};

pub(crate) struct MenuScreenPlugin;

impl Plugin for MenuScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayScreenState::Menu), spawn_menu);

        app.add_systems(
            Update,
            (
                manage_menu_button_input,
                manage_menu_keyboard_input.run_if(on_event::<KeyboardInput>()),
            )
                .run_if(in_state(GameplayScreenState::Menu)),
        );
    }
}
