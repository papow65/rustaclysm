use application_state::ApplicationState;
use bevy::prelude::{StateSet as _, SubStates};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(ApplicationState = ApplicationState::Gameplay)]
pub enum GameplayTransitionState {
    #[default]
    Loading,
    Loaded,
    Unloading,
}

impl GameplayTransitionState {
    #[must_use]
    pub fn in_transition(self) -> bool {
        self != Self::Loaded
    }
}
