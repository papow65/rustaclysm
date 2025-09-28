use crate::ApplicationState;
use bevy::prelude::{App, AppExtStates as _, Plugin};
use util::log_transition_plugin;

pub struct ApplicationStatePlugin;

impl Plugin for ApplicationStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(log_transition_plugin::<ApplicationState>);

        app.init_state::<ApplicationState>();
    }
}
