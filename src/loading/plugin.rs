use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct LoadingIndicatorPlugin;

impl Plugin for LoadingIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgressScreenState::Loading), spawn_loading);

        app.add_systems(
            Update,
            (
                start_gameplay,
                finish_loading.run_if(resource_exists::<Explored>()),
            )
                .run_if(in_state(ProgressScreenState::Loading)),
        );

        app.add_systems(
            OnExit(ProgressScreenState::Loading),
            despawn::<ProgressScreenState>,
        );
    }
}
