use bevy::prelude::States;

/// For `ApplicationState` transitions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum ProgressScreenState {
    Loading,
    #[default]
    Complete,
}
