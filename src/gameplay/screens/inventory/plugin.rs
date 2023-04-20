use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct InventoryScreenPlugin;

impl Plugin for InventoryScreenPlugin {
    fn build(&self, app: &mut App) {
        // startup
        app.add_systems((spawn_inventory,).in_schedule(OnEnter(GameplayScreenState::Inventory)));

        // every frame
        app.add_systems(
            (manage_inventory_keyboard_input,).in_set(OnUpdate(GameplayScreenState::Inventory)),
        )
        .add_systems(
            (manage_inventory_button_input, run_behavior_schedule)
                .chain()
                .in_set(OnUpdate(GameplayScreenState::Inventory)),
        );

        // shutdown
        app.add_systems((despawn_inventory,).in_schedule(OnExit(GameplayScreenState::Inventory)));
    }
}
