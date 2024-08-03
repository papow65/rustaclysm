use bevy::{
    dev_tools::states::log_transitions,
    prelude::{App, States, Update},
};

pub(crate) fn log_transition_plugin<S: States>(app: &mut App) {
    if cfg!(debug_assertions) {
        app.add_systems(Update, log_transitions::<S>);
    }
}
