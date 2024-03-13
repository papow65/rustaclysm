use crate::prelude::*;
use bevy::prelude::*;

/** Plugin for the screen-independent HUD: the manual and the sidebar */
pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            (create_hud_defaults, (spawn_manual, spawn_sidebar)).chain(),
        );

        app.add_systems(
            Update,
            (
                manage_inventory_keyboard_input,
                clear_inventory.pipe(update_inventory),
                (
                    manage_inventory_button_input,
                    manage_inventory_mouse_input,
                    run_behavior_schedule.pipe(run_behavior_display_schedule),
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
