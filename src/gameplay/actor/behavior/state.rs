use crate::gameplay::{FocusState, GameplayScreenState};
use bevy::prelude::ComputedStates;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct BehaviorState;

impl ComputedStates for BehaviorState {
    type SourceStates = (GameplayScreenState, FocusState);

    fn compute((gameplay_screen_state, focus_state): Self::SourceStates) -> Option<Self> {
        match gameplay_screen_state {
            GameplayScreenState::Crafting | GameplayScreenState::Inventory => true,
            GameplayScreenState::Base => focus_state == FocusState::Normal,
            _ => false,
        }
        .then_some(Self)
    }
}
