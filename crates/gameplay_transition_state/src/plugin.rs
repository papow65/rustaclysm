use crate::GameplayTransitionState;
use bevy::prelude::{App, AppExtStates as _, Plugin};
use util::log_transition_plugin;

pub struct GameplayTransitionStatePlugin;

impl Plugin for GameplayTransitionStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(log_transition_plugin::<GameplayTransitionState>);

        app.add_sub_state::<GameplayTransitionState>();
        app.enable_state_scoped_entities::<GameplayTransitionState>();
    }
}
