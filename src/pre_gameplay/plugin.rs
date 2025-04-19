use crate::pre_gameplay::systems::{spawn_pre_gameplay_camera, start_gameplay};
use application_state::ApplicationState;
use bevy::prelude::{App, IntoScheduleConfigs as _, OnEnter, Plugin, Update, in_state};

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
