use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct InventoryScreenPlugin;

impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        // startup
        app.add_systems(OnEnter(GameplayScreenState::Inventory), spawn_inventory);

        // every frame
        app.add_systems(
            Update,
            manage_inventory_keyboard_input.run_if(in_state(GameplayScreenState::Inventory)),
        )
        .add_systems(
            Update,
            clear_inventory
                .pipe(update_inventory)
                .after(UpdateSet::FlushEffects)
                .run_if(in_state(GameplayScreenState::Inventory)),
        )
        .add_systems(
            Update,
            (manage_inventory_button_input, run_behavior_schedule)
                .chain()
                .run_if(in_state(GameplayScreenState::Inventory)),
        );

        // shutdown
        app.add_systems(OnExit(GameplayScreenState::Inventory), despawn_inventory);
    }
}
