use crate::systems::start_gameplay;
use application_state::ApplicationState;
use bevy::prelude::{App, IntoScheduleConfigs as _, Plugin, Update, in_state};

/// This plugin launches the gameplay when the required resources become available.
pub struct PreGameplayPlugin;

impl Plugin for PreGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            start_gameplay.run_if(in_state(ApplicationState::PreGameplay)),
        );
    }
}
