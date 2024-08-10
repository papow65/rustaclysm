use bevy::prelude::States;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum ApplicationState {
    #[default]
    Startup,
    MainMenu,
    Gameplay,
}
