use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct LoadingIndicatorPlugin;

impl Plugin for LoadingIndicatorPlugin {
    fn build(&self, app: &mut App) {
        // startup
        app.add_systems((spawn_loading,).in_schedule(OnEnter(ProgressScreenState::Loading)));

        // every frame
        app.add_systems(
            (
                start_gameplay,
                finish_loading.run_if(resource_exists::<Explored>()),
            )
                .in_set(OnUpdate(ProgressScreenState::Loading)),
        );

        // shutdown
        app.add_systems((despawn_loading,).in_schedule(OnExit(ProgressScreenState::Loading)));
    }
}
