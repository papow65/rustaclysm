use bevy::prelude::States;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum GameplayScreenState {
    #[default]
    Loading,
    Base,
    Character,
    Inventory,
    //Crafting, // TODO
    /** Not the main menu */
    Menu,
    //Saving, // TODO
    /** ApplicationState is not Gameplay */
    Inapplicable,
}
