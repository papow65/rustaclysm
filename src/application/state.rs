use bevy::prelude::States;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum ApplicationState {
    Gameplay,
    #[default]
    MainMenu,
}
