use crate::{LoadingScreenPlugin, UnloadingScreenPlugin};
use bevy::prelude::{App, Plugin};
use gameplay_transition_state::GameplayTransitionStatePlugin;

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayTransitionStatePlugin,
            LoadingScreenPlugin,
            UnloadingScreenPlugin,
        ));
    }
}
