use super::systems::{
    clear_crafting_screen, manage_crafting_keyboard_input, remove_crafting_resource,
    spawn_crafting_screen, update_crafting_screen,
};
use crate::prelude::{despawn, loop_behavior_and_refresh, GameplayScreenState};
use bevy::{
    input::keyboard::KeyboardInput,
    prelude::{
        in_state, on_event, App, IntoSystem, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
    },
};

pub(crate) struct CraftingScreenPlugin;

impl Plugin for CraftingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Crafting),
            spawn_crafting_screen,
        );

        app.add_systems(
            Update,
            (
                manage_crafting_keyboard_input.run_if(on_event::<KeyboardInput>()),
                clear_crafting_screen.pipe(update_crafting_screen),
                loop_behavior_and_refresh(),
            )
                .run_if(in_state(GameplayScreenState::Crafting)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Crafting),
            (despawn::<GameplayScreenState>, remove_crafting_resource),
        );
    }
}
