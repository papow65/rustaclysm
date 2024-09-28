use crate::gameplay::screens::inventory::systems::{
    clear_inventory, create_inventory_key_bindings, manage_inventory_button_input,
    refresh_inventory, remove_inventory_resource, spawn_inventory,
};
use crate::gameplay::GameplayScreenState;
use bevy::prelude::{
    in_state, App, IntoSystem, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
};

pub(crate) struct InventoryScreenPlugin;

impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Inventory),
            (spawn_inventory, create_inventory_key_bindings),
        );

        app.add_systems(
            Update,
            (
                clear_inventory.pipe(refresh_inventory),
                manage_inventory_button_input,
            )
                .run_if(in_state(GameplayScreenState::Inventory)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Inventory),
            remove_inventory_resource,
        );
    }
}
