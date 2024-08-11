use crate::gameplay::{FocusState, GameplayScreenState};
use crate::loading::ProgressScreenState;
use bevy::prelude::ComputedStates;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct BehaviorState;

impl ComputedStates for BehaviorState {
    type SourceStates = (ProgressScreenState, GameplayScreenState, FocusState);

    fn compute(source_states: Self::SourceStates) -> Option<Self> {
        let (progress_screen_state, gameplay_screen_state, focus_state) = source_states;
        (progress_screen_state == ProgressScreenState::Complete
            && match gameplay_screen_state {
                GameplayScreenState::Crafting | GameplayScreenState::Inventory => true,
                GameplayScreenState::Base => focus_state == FocusState::Normal,
                _ => false,
            })
        .then_some(Self)
    }
}
