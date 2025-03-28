use crate::gameplay::GameplayScreenState;
use crate::gameplay::screens::inventory::systems::{
    InventoryButton, clear_inventory, create_inventory_key_bindings, create_inventory_system,
    refresh_inventory, remove_inventory_resource, spawn_inventory,
};
use bevy::prelude::{
    App, In, IntoScheduleConfigs as _, IntoSystem as _, OnEnter, OnExit, Plugin, Update, in_state,
};
use hud::manage_button_input;

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
                create_inventory_system.pipe(clear_inventory.pipe(refresh_inventory)),
                manage_button_input::<In<InventoryButton>>,
            )
                .run_if(in_state(GameplayScreenState::Inventory)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Inventory),
            remove_inventory_resource,
        );
    }
}
