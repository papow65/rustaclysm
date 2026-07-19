use crate::screens::transitioning::systems::to_base_screen;
use bevy::prelude::{App, IntoScheduleConfigs as _, OnEnter, Plugin, in_state};
use gameplay_screen_state::GameplayScreenState;
use gameplay_transition_state::GameplayTransitionState;

pub(crate) struct TransitioningScreenPlugin;

impl Plugin for TransitioningScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayTransitionState::Loaded),
            to_base_screen.run_if(in_state(GameplayScreenState::Transitioning)),
        );
    }
}
