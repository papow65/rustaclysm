use application_state::ApplicationState;
use bevy::prelude::{
    App, Camera3d, Commands, Entity, IntoScheduleConfigs as _, NextState, Plugin, Query, ResMut,
    Update, With, debug, in_state,
};
use gameplay_transition_state::GameplayTransitionState;

pub(crate) struct UnloadingScreenPlugin;

impl Plugin for UnloadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            finish_unloading.run_if(in_state(GameplayTransitionState::Unloading)),
        );
    }
}

fn finish_unloading(
    mut commands: Commands,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    debug!("Unloading complete");
    next_application_state.set(ApplicationState::MainMenu);

    for camera in &cameras {
        commands.entity(camera).despawn();
    }
}
