use crate::application::ApplicationState;
use crate::pre_gameplay::systems::{spawn_pre_gameplay_camera, start_gameplay};
use bevy::prelude::{in_state, App, IntoSystemConfigs, OnEnter, Plugin, Update};

/// This plugin launches the gameplay when the required resources become available.
pub(crate) struct PreGameplayPlugin;

impl Plugin for PreGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::PreGameplay),
            spawn_pre_gameplay_camera,
        );

        app.add_systems(
            Update,
            start_gameplay.run_if(in_state(ApplicationState::PreGameplay)),
        );
    }
}
