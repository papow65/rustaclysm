use crate::{GameplayReadiness, GameplayScreenState};
use bevy::prelude::{
    App, IntoScheduleConfigs as _, NextState, Plugin, ResMut, Update, debug, in_state,
};
use gameplay_transition_state::GameplayTransitionState;

pub(crate) struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            finish_loading.run_if(in_state(GameplayTransitionState::Loading)),
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn finish_loading(
    mut next_gameplay_transition_state: ResMut<NextState<GameplayTransitionState>>,
    mut next_gameplay_screen_state: ResMut<NextState<GameplayScreenState>>,
    gameplay_readiness: GameplayReadiness,
) {
    if gameplay_readiness.ready_to_run() {
        debug!("Loading complete");
        next_gameplay_transition_state.set(GameplayTransitionState::Loaded);
        next_gameplay_screen_state.set(GameplayScreenState::Base);
    }
}
