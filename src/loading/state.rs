use crate::gameplay::GameplayScreenState;
use bevy::prelude::ComputedStates;

/// For `ApplicationState` transitions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct LoadingState;

impl ComputedStates for LoadingState {
    type SourceStates = GameplayScreenState;

    fn compute(gameplay_screen_state: Self::SourceStates) -> Option<Self> {
        (gameplay_screen_state == GameplayScreenState::Loading).then_some(Self)
    }
}
