use crate::loading::LoadingIndicatorState;
use application_state::ApplicationState;
use bevy::prelude::ComputedStates;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct BackgroundState;

impl ComputedStates for BackgroundState {
    type SourceStates = (ApplicationState, Option<LoadingIndicatorState>);

    fn compute((application_state, loading_state): Self::SourceStates) -> Option<Self> {
        (application_state == ApplicationState::MainMenu || loading_state.is_some()).then_some(Self)
    }
}
