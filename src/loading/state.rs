use crate::{application::ApplicationState, gameplay::GameplayScreenState};
use bevy::prelude::ComputedStates;

/// For `ApplicationState` transitions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct LoadingState;

impl ComputedStates for LoadingState {
    type SourceStates = (ApplicationState, Option<GameplayScreenState>);

    fn compute((application_state, gameplay_screen_state): Self::SourceStates) -> Option<Self> {
        (application_state == ApplicationState::PreGameplay
            || gameplay_screen_state == Some(GameplayScreenState::Loading))
        .then_some(Self)
    }
}
