use application_state::ApplicationState;
use bevy::prelude::{StateSet as _, SubStates};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(ApplicationState = ApplicationState::Gameplay)]
pub(crate) enum GameplayScreenState {
    #[default]
    Transitioning,

    /// Walking around, etc.
    Base,

    //Character, // TODO
    Inventory,

    Crafting,

    Quality,

    Tool,

    /// Different from the main menu
    Menu,

    //Saving, // TODO
    Death,
}
