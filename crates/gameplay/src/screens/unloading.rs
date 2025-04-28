use crate::GameplayScreenState;
use application_state::ApplicationState;
use bevy::prelude::{
    App, Camera3d, Commands, Entity, IntoScheduleConfigs as _, Local, NextState, Plugin, Query,
    ResMut, Update, With, debug, in_state,
};
use std::num::Wrapping;

pub(crate) struct UnloadingScreenPlugin;

impl Plugin for UnloadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            finish_unloading.run_if(in_state(GameplayScreenState::Unloading)),
        );
    }
}

fn finish_unloading(
    mut commands: Commands,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    cameras: Query<Entity, With<Camera3d>>,
    mut counter: Local<Wrapping<u8>>,
) {
    *counter += 1;
    if counter.0 % 10 != 0 {
        debug!("Unloading later");
        return;
    }

    debug!("Unloading complete");
    next_application_state.set(ApplicationState::MainMenu);

    for camera in &cameras {
        commands.entity(camera).despawn();
    }
}
