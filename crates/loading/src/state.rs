use application_state::ApplicationState;
use bevy::prelude::ComputedStates;
use gameplay_transition_state::GameplayTransitionState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LoadingIndicatorState;

impl ComputedStates for LoadingIndicatorState {
    type SourceStates = (ApplicationState, Option<GameplayTransitionState>);

    fn compute((application_state, gameplay_transition_state): Self::SourceStates) -> Option<Self> {
        (application_state == ApplicationState::PreGameplay
            || gameplay_transition_state.is_some_and(GameplayTransitionState::in_transition))
        .then_some(Self)
    }
}
