use crate::gameplay::screens::crafting::systems::{
    clear_crafting_screen, create_crafting_key_bindings, create_start_craft_system,
    refresh_crafting_screen, remove_crafting_resource, spawn_crafting_screen,
};
use crate::gameplay::GameplayScreenState;
use bevy::prelude::{
    in_state, App, IntoSystem as _, IntoSystemConfigs as _, OnEnter, OnExit, Plugin, Update,
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
            create_start_craft_system
                .pipe(clear_crafting_screen.pipe(refresh_crafting_screen))
                .run_if(in_state(GameplayScreenState::Crafting)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Crafting),
            remove_crafting_resource,
        );
    }
}
