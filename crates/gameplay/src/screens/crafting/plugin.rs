use crate::GameplayScreenState;
use crate::screens::crafting::systems::{
    clear_crafting_screen, create_crafting_key_bindings, create_start_craft_system,
    refresh_crafting_screen, remove_crafting_resource, spawn_crafting_screen,
};
use bevy::prelude::{
    App, IntoScheduleConfigs as _, IntoSystem as _, OnEnter, OnExit, Plugin, Update, in_state,
};

pub(crate) struct CraftingScreenPlugin;

impl Plugin for CraftingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Crafting),
            (
                create_start_craft_system.pipe(spawn_crafting_screen),
                create_crafting_key_bindings,
            ),
        );

        app.add_systems(
            Update,
            clear_crafting_screen
                .pipe(refresh_crafting_screen)
                .run_if(in_state(GameplayScreenState::Crafting)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Crafting),
            remove_crafting_resource,
        );
    }
}
