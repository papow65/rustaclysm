use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct LoadingIndicatorPlugin;

impl Plugin for LoadingIndicatorPlugin {
    fn build(&self, app: &mut App) {
        // startup
        app.add_systems(OnEnter(ProgressScreenState::Loading), spawn_loading);

        // every frame
        app.add_systems(
            Update,
            (
                start_gameplay,
                finish_loading.run_if(resource_exists::<Explored>()),
            )
                .run_if(in_state(ProgressScreenState::Loading)),
        );

        // shutdown
        app.add_systems(OnExit(ProgressScreenState::Loading), despawn_loading);
    }
}
