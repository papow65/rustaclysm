use super::systems::{manage_death_button_input, manage_death_keyboard_input, spawn_death_screen};
use crate::prelude::{despawn, GameplayScreenState};
use bevy::prelude::{in_state, App, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update};

pub(crate) struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayScreenState::Death), spawn_death_screen);

        app.add_systems(
            Update,
            (manage_death_keyboard_input, manage_death_button_input)
                .run_if(in_state(GameplayScreenState::Death)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Death),
            despawn::<GameplayScreenState>,
        );
    }
}
