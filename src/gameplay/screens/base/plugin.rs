use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_mouse_input.before(/* process zooming input */ update_camera_offset),
                (
                    manage_keyboard_input
                        .before(/* process examining input */ update_camera_base)
                        .before(/* process zooming input */ update_camera_offset),
                    run_behavior_schedule,
                )
                    .chain(),
            )
                .run_if(in_state(GameplayScreenState::Base)),
        );
    }
}
