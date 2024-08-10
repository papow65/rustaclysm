use crate::gameplay::screens::base::systems::{
    manage_keyboard_input, manage_mouse_button_input, manage_mouse_scroll_input,
    update_camera_offset,
};
use crate::gameplay::GameplayScreenState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::{in_state, on_event, App, IntoSystemConfigs, Plugin, Update};

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_mouse_scroll_input
                    .run_if(on_event::<MouseWheel>())
                    .before(update_camera_offset),
                manage_mouse_button_input.run_if(on_event::<MouseMotion>()),
                manage_keyboard_input
                    .run_if(on_event::<KeyboardInput>())
                    .before(update_camera_offset),
            )
                .run_if(in_state(GameplayScreenState::Base)),
        );
    }
}
