use crate::screens::base::systems::{
    create_base_key_bindings, manage_mouse_button_input, manage_mouse_scroll_input,
    trigger_refresh, update_camera_offset,
};
use crate::{GameplayScreenState, PlayerActionState};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::{
    App, IntoScheduleConfigs as _, OnEnter, OnExit, Plugin, Update, in_state, on_message,
};
use gameplay_location::CardinalDirection;
use strum::VariantArray as _;

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameplayScreenState::Base), create_base_key_bindings);

        app.add_systems(
            Update,
            (
                manage_mouse_scroll_input
                    .run_if(on_message::<MouseWheel>)
                    .before(update_camera_offset),
                manage_mouse_button_input.run_if(on_message::<MouseMotion>),
            )
                .run_if(in_state(GameplayScreenState::Base)),
        );

        for &direction in CardinalDirection::VARIANTS {
            let peeking = PlayerActionState::Peeking { direction };
            app.add_systems(OnEnter(peeking.clone()), trigger_refresh);
            app.add_systems(OnExit(peeking), trigger_refresh);
        }
    }
}
