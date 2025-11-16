use crate::screens::inventory::systems::{
    InventoryButton, adapt_to_item_selection, create_inventory_key_bindings,
    create_inventory_system, refresh_inventory, remove_inventory_resource, spawn_inventory,
};
use crate::{GameplayScreenState, RefreshAfterBehavior};
use bevy::prelude::{
    App, In, IntoScheduleConfigs as _, IntoSystem as _, OnEnter, OnExit, Plugin, Update, With,
    in_state, on_message,
};
use hud::manage_button_input;
use selection_list::{SelectionList, selection_list_plugin};

pub(crate) struct InventoryScreenPlugin;

impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        selection_list_plugin::<_, _, _, With<SelectionList>>(
            app,
            GameplayScreenState::Inventory,
            "select item",
            adapt_to_item_selection,
        );

        app.add_systems(
            OnEnter(GameplayScreenState::Inventory),
            (
                (
                    create_inventory_system.pipe(spawn_inventory),
                    refresh_inventory,
                )
                    .chain(),
                create_inventory_key_bindings,
            ),
        );

        app.add_systems(
            Update,
            (
                refresh_inventory.run_if(on_message::<RefreshAfterBehavior>),
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
