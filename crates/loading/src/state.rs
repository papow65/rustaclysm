use application_state::ApplicationState;
use bevy::prelude::ComputedStates;
use gameplay::GameplayScreenState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LoadingIndicatorState;

impl ComputedStates for LoadingIndicatorState {
    type SourceStates = (ApplicationState, Option<GameplayScreenState>);

    fn compute((application_state, gameplay_screen_state): Self::SourceStates) -> Option<Self> {
        (application_state == ApplicationState::PreGameplay
            || gameplay_screen_state == Some(GameplayScreenState::Loading))
        .then_some(Self)
    }
}
