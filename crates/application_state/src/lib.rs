use bevy::prelude::States;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, States)]
pub enum ApplicationState {
    #[default]
    Startup,
    MainMenu,
    PreGameplay,
    Gameplay,
}
