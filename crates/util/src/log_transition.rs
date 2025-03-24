use bevy::dev_tools::states::log_transitions;
use bevy::prelude::{App, States, Update};

pub fn log_transition_plugin<S: States>(app: &mut App) {
    if cfg!(debug_assertions) {
        app.add_systems(Update, log_transitions::<S>);
    }
}
