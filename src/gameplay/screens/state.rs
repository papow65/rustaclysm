use crate::application::ApplicationState;
use bevy::prelude::{StateSet as _, SubStates};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(ApplicationState = ApplicationState::Gameplay)]
pub(crate) enum GameplayScreenState {
    #[default]
    Loading,
    Base,
    //Character, // TODO
    Inventory,
    Crafting,
    /// Different from the main menu
    Menu,
    //Saving, // TODO
    Death,
}
