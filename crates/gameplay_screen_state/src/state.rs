use application_state::ApplicationState;
use bevy::prelude::SubStates;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(ApplicationState = ApplicationState::Gameplay)]
pub enum GameplayScreenState {
    #[default]
    Transitioning,

    /// Walking around, etc.
    Base,

    //Character, // TODO
    Inventory,

    Crafting,

    Quality,

    Tool,

    Waiting,

    /// Different from the main menu
    Menu,

    //Saving, // TODO
    Death,
}

impl GameplayScreenState {
    #[must_use]
    pub const fn allow_behavior(self) -> bool {
        !matches!(self, Self::Transitioning | Self::Menu | Self::Death)
    }
}
