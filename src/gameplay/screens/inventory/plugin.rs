use super::systems::{
    clear_inventory, manage_inventory_button_input, manage_inventory_keyboard_input,
    manage_inventory_mouse_input, remove_inventory_resource, spawn_inventory, update_inventory,
};
use crate::prelude::{despawn, loop_behavior_and_refresh, GameplayScreenState};
use bevy::prelude::{
    in_state, App, IntoSystem, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
};

pub(crate) struct InventoryScreenPlugin;

impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayScreenState::Inventory), spawn_inventory);

        app.add_systems(
            Update,
            (
                manage_inventory_keyboard_input,
                clear_inventory.pipe(update_inventory),
                (
                    manage_inventory_button_input,
                    manage_inventory_mouse_input,
                    loop_behavior_and_refresh(),
                )
                    .chain(),
            )
                .run_if(in_state(GameplayScreenState::Inventory)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Inventory),
            (despawn::<GameplayScreenState>, remove_inventory_resource),
        );
    }
}
