use crate::GameplayScreenState;
use crate::screens::base::systems::{create_base_key_bindings, trigger_refresh};
use bevy::prelude::{App, IntoScheduleConfigs as _, OnEnter, OnExit, Plugin, Update, in_state};
use gameplay_camera::{CameraPlugin, manage_camera_offset};
use gameplay_location::CardinalDirection;
use gameplay_player::PlayerActionState;
use strum::VariantArray as _;

pub(crate) struct BaseScreenPlugin;

impl Plugin for BaseScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin);

        app.add_systems(OnEnter(GameplayScreenState::Base), create_base_key_bindings);

        app.add_systems(
            Update,
            manage_camera_offset().run_if(in_state(GameplayScreenState::Base)),
        );

        for &direction in CardinalDirection::VARIANTS {
            let peeking = PlayerActionState::Peeking { direction };
            app.add_systems(OnEnter(peeking.clone()), trigger_refresh);
            app.add_systems(OnExit(peeking), trigger_refresh);
        }
    }
}
