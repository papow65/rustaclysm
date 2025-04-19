use crate::GameplayScreenState;
use crate::screens::death::systems::{
    create_death_screen_key_bindings, create_main_menu_system, spawn_death_screen,
};
use bevy::prelude::{App, IntoSystem as _, OnEnter, Plugin};

pub(crate) struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Death),
            (
                create_main_menu_system.pipe(spawn_death_screen),
                create_death_screen_key_bindings,
            ),
        );
    }
}
