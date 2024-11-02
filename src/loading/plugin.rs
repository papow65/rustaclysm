use crate::gameplay::Explored;
use crate::loading::systems::{finish_loading, spawn_loading};
use crate::loading::LoadingState;
use crate::util::log_transition_plugin;
use bevy::prelude::{
    in_state, resource_exists, App, AppExtStates, Condition, IntoSystemConfigs, OnEnter, Plugin,
    Update,
};

pub(crate) struct LoadingIndicatorPlugin;

impl Plugin for LoadingIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<LoadingState>();
        app.enable_state_scoped_entities::<LoadingState>();
        app.add_plugins(log_transition_plugin::<LoadingState>);

        app.add_systems(OnEnter(LoadingState), spawn_loading);

        app.add_systems(
            Update,
            finish_loading.run_if(in_state(LoadingState).and(resource_exists::<Explored>)),
        );
    }
}
