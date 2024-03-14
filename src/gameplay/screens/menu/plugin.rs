use super::systems::{manage_menu_button_input, manage_menu_keyboard_input, spawn_menu};
use crate::prelude::{despawn, GameplayScreenState};
use bevy::prelude::{in_state, App, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update};

pub(crate) struct MenuScreenPlugin;

impl Plugin for MenuScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayScreenState::Menu), spawn_menu);

        app.add_systems(
            Update,
            (manage_menu_button_input, manage_menu_keyboard_input)
                .run_if(in_state(GameplayScreenState::Menu)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Menu),
            despawn::<GameplayScreenState>,
        );
    }
}
