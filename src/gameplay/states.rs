use bevy::prelude::States;

/** Conceptually, this is a child state of `ApplicationState::Gameplay` */
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum GameplayScreenState {
    Base,
    Character,
    Inventory,
    //Crafting, // TODO
    /** Different from the main menu */
    Menu,
    //Saving, // TODO
    /** When not ApplicationState::Gameplay */
    #[default]
    Inapplicable,
}