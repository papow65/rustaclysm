use bevy::prelude::States;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum ApplicationState {
    Gameplay,
    #[default]
    MainMenu,
}

/** For `ApplicationState` transitions */
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum ProgressScreenState {
    Loading,
    #[default]
    Complete,
}
