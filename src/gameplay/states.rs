use bevy::prelude::States;

/** Conceptually, this is a child state of `ApplicationState::Gameplay` */
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum GameplayScreenState {
    // Loading, // TODO
    Base,
    Character,
    Inventory,
    //Crafting, // TODO
    /** Not the main menu */
    Menu,
    //Saving, // TODO
    /** ApplicationState is not Gameplay */
    #[default]
    Inapplicable,
}
