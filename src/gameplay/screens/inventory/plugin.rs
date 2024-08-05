use crate::gameplay::screens::inventory::systems::{
    clear_inventory, manage_inventory_button_input, manage_inventory_keyboard_input,
    refresh_inventory, remove_inventory_resource, spawn_inventory,
};
use crate::prelude::{loop_behavior_and_refresh, GameplayScreenState};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{
    in_state, on_event, App, IntoSystem, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
};

pub(crate) struct InventoryScreenPlugin;

impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayScreenState::Inventory), spawn_inventory);

        app.add_systems(
            Update,
            (
                manage_inventory_keyboard_input.run_if(on_event::<KeyboardInput>()),
                clear_inventory.pipe(refresh_inventory),
                (manage_inventory_button_input, loop_behavior_and_refresh()).chain(),
            )
                .run_if(in_state(GameplayScreenState::Inventory)),
        );

        app.add_systems(
            OnExit(GameplayScreenState::Inventory),
            remove_inventory_resource,
        );
    }
}
