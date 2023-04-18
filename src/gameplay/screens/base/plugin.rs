use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        // executed every frame
        app.add_system(
            manage_mouse_input
                .before(update_camera_base)
                .before(update_camera_offset)
                .run_if(in_state(GameplayScreenState::Base)),
        )
        .add_systems(
            (
                manage_keyboard_input
                    .before(update_camera_base)
                    .before(update_camera_offset),
                run_behavior_schedule,
            )
                .chain()
                .in_set(OnUpdate(GameplayScreenState::Base)),
        );
    }
}
