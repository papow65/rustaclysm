use crate::ActiveSav;
use application_state::ApplicationState;
use bevy::prelude::{App, Commands, OnExit, Plugin};

pub struct ActiveSavPlugin;

impl Plugin for ActiveSavPlugin {
    fn build(&self, app: &mut App) {
        // ActiveSav is created in the main menu
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_active_sav);
    }
}

fn remove_active_sav(mut commands: Commands) {
    commands.remove_resource::<ActiveSav>();
}
