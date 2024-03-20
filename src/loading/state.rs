use bevy::prelude::States;

/** For `ApplicationState` transitions */
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
pub(crate) enum ProgressScreenState {
    Loading,
    #[default]
    Complete,
}
