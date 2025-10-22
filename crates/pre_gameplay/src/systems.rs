use application_state::ApplicationState;
use bevy::prelude::{NextState, ResMut};
use gameplay::GameplayReadiness;

#[expect(clippy::needless_pass_by_value)]
pub(super) fn start_gameplay(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    gameplay_readiness: GameplayReadiness,
) {
    if gameplay_readiness.ready_to_load() {
        next_application_state.set(ApplicationState::Gameplay);
    }
}
