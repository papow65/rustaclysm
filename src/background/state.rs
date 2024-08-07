use crate::{application::ApplicationState, loading::ProgressScreenState};
use bevy::prelude::ComputedStates;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct BackgroundState;

impl ComputedStates for BackgroundState {
    type SourceStates = (ApplicationState, ProgressScreenState);

    fn compute((application_state, loading_state): Self::SourceStates) -> Option<Self> {
        (application_state == ApplicationState::MainMenu
            || loading_state == ProgressScreenState::Loading)
            .then_some(Self)
    }
}
