use crate::GameplayTransitionState;
use bevy::prelude::{App, AppExtStates as _, Camera, Camera3d, OnEnter, Plugin, Query, With, info};
use util::log_transition_plugin;

pub struct GameplayTransitionStatePlugin;

impl Plugin for GameplayTransitionStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(log_transition_plugin::<GameplayTransitionState>);

        app.add_sub_state::<GameplayTransitionState>();
        app.enable_state_scoped_entities::<GameplayTransitionState>();

        app.add_systems(OnEnter(GameplayTransitionState::Loaded), enable_3d_camera);
    }
}

fn enable_3d_camera(mut cameras: Query<&mut Camera, With<Camera3d>>) {
    cameras
        .single_mut()
        .expect("3D camera should be present")
        .is_active = true;
    info!("Enabled 3D camera");
}
