use crate::gameplay::screens::crafting::systems::{
    clear_crafting_screen, create_crafting_key_bindings, manage_crafting_button_input,
    refresh_crafting_screen, remove_crafting_resource, spawn_crafting_screen,
};
use crate::gameplay::GameplayScreenState;
use bevy::prelude::{
    in_state, App, IntoSystem, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
};

pub(crate) struct CraftingScreenPlugin;

impl Plugin for CraftingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Crafting),
            (spawn_crafting_screen, create_crafting_key_bindings),
        );

        app.add_systems(
            Update,
            (
                manage_crafting_button_input,
                clear_crafting_screen.pipe(refresh_crafting_screen),
            )
                .run_if(in_state(GameplayScreenState::Crafting)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Crafting),
            remove_crafting_resource,
        );
    }
}
