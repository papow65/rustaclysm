use super::{
    focus::FocusState,
    systems::{
        manage_keyboard_input, manage_mouse_button_input, manage_mouse_scroll_input,
        update_camera_base, update_camera_offset, update_focus_cursor_visibility,
    },
};
use crate::prelude::{loop_behavior_and_refresh, update_visibility, GameplayScreenState};
use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseMotion, MouseWheel},
    },
    prelude::{
        in_state, not, on_event, resource_exists_and_changed, App, AppExtStates, IntoSystemConfigs,
        Plugin, State, Update,
    },
};

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(FocusState::Normal);

        app.add_systems(
            Update,
            (
                manage_mouse_scroll_input
                    .run_if(on_event::<MouseWheel>())
                    .before(update_camera_offset),
                manage_mouse_button_input.run_if(on_event::<MouseMotion>()),
                (
                    manage_keyboard_input
                        .run_if(on_event::<KeyboardInput>())
                        .before(update_camera_offset),
                    (
                        loop_behavior_and_refresh().run_if(in_state(FocusState::Normal)),
                        (
                            update_focus_cursor_visibility,
                            update_visibility.run_if(not(in_state(FocusState::Normal))),
                            update_camera_base,
                        )
                            .chain()
                            .run_if(resource_exists_and_changed::<State<FocusState>>),
                    ),
                )
                    .chain(),
            )
                .run_if(in_state(GameplayScreenState::Base)),
        );
    }
}
