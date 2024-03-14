use super::systems::{manage_keyboard_input, manage_mouse_input};
use crate::prelude::{behavior_systems, update_camera_offset, GameplayScreenState};
use bevy::prelude::{in_state, App, IntoSystemConfigs, Plugin, Update};

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_mouse_input.before(/* process zooming input */ update_camera_offset),
                (
                    manage_keyboard_input
                        .before(/* process zooming input */ update_camera_offset),
                    behavior_systems(),
                )
                    .chain(),
            )
                .run_if(in_state(GameplayScreenState::Base)),
        );
    }
}
