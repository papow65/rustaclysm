use crate::common::log_transition_plugin;
use crate::gameplay::{Explored, Infos, RelativeSegments};
use crate::loading::systems::{finish_loading, spawn_loading, start_gameplay};
use crate::loading::ProgressScreenState;
use bevy::prelude::{
    in_state, resource_exists, App, AppExtStates, Condition, IntoSystemConfigs, OnEnter, Plugin,
    Update,
};

pub(crate) struct LoadingIndicatorPlugin;

impl Plugin for LoadingIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(ProgressScreenState::Complete);
        app.enable_state_scoped_entities::<ProgressScreenState>();

        app.add_plugins((log_transition_plugin::<ProgressScreenState>,));

        app.add_systems(OnEnter(ProgressScreenState::Loading), spawn_loading);

        app.add_systems(
            Update,
            (
                start_gameplay.run_if(resource_exists::<Infos>),
                finish_loading.run_if(
                    resource_exists::<Explored>.and_then(resource_exists::<RelativeSegments>),
                ),
            )
                .run_if(in_state(ProgressScreenState::Loading)),
        );
    }
}
