use application_state::ApplicationState;
use bevy::prelude::{NextState, ResMut};
use gameplay::GameplayReadiness;
use gameplay_transition_state::GameplayTransitionState;

#[expect(clippy::needless_pass_by_value)]
pub(super) fn start_gameplay(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_transition_state: ResMut<NextState<GameplayTransitionState>>,
    gameplay_readiness: GameplayReadiness,
) {
    if gameplay_readiness.ready_to_load() {
        next_application_state.set(ApplicationState::Gameplay);
        next_gameplay_transition_state.set(GameplayTransitionState::Loading);
    }
}
