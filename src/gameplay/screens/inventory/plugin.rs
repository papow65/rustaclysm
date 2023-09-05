use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct InventoryScreenPlugin;

impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayScreenState::Inventory), spawn_inventory);

        app.add_systems(
            Update,
            (
                manage_inventory_keyboard_input,
                clear_inventory.pipe(update_inventory),
                (manage_inventory_button_input, run_behavior_schedule).chain(),
            )
                .run_if(in_state(GameplayScreenState::Inventory)),
        );

        app.add_systems(OnExit(GameplayScreenState::Inventory), despawn_inventory);
    }
}
