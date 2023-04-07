use crate::prelude::*;
use bevy::prelude::*;

pub(crate) struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        // startup
        app.add_systems((spawn_loading,).in_schedule(OnEnter(GameplayScreenState::Loading)));

        // every frame
        app.add_system(finish_loading.in_set(OnUpdate(GameplayScreenState::Loading)));

        // shutdown
        app.add_systems((despawn_loading,).in_schedule(OnExit(GameplayScreenState::Loading)));
    }
}
