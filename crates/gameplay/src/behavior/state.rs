use crate::{FocusState, GameplayScreenState};
use bevy::prelude::ComputedStates;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct BehaviorState;

impl ComputedStates for BehaviorState {
    type SourceStates = (GameplayScreenState, FocusState);

    fn compute(source_states: Self::SourceStates) -> Option<Self> {
        let (gameplay_transition_state, focus_state) = source_states;
        (match gameplay_transition_state {
            GameplayScreenState::Crafting | GameplayScreenState::Inventory => true,
            GameplayScreenState::Base => focus_state == FocusState::Normal,
            _ => false,
        })
        .then_some(Self)
    }
}
