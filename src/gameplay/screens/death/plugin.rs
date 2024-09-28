use crate::gameplay::screens::death::systems::{
    create_death_screen_key_bindings, manage_death_button_input, spawn_death_screen,
};
use crate::gameplay::GameplayScreenState;
use bevy::prelude::{in_state, App, IntoSystemConfigs, OnEnter, Plugin, Update};

pub(crate) struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Death),
            (spawn_death_screen, create_death_screen_key_bindings),
        );

        app.add_systems(
            Update,
            manage_death_button_input.run_if(in_state(GameplayScreenState::Death)),
        );
    }
}
