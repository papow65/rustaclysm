#[allow(unused_imports)]
use bevy::{
    dev_tools::states::log_transitions,
    prelude::{App, States, Update},
};

#[allow(unused_variables)]
pub(crate) fn log_transition_plugin<S: States>(app: &mut App) {
    #[cfg(debug_assertions)]
    app.add_systems(Update, log_transitions::<S>);
}
