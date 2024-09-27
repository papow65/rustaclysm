use crate::application::ApplicationState;
use crate::gameplay::sidebar::resources::{despawn_sidebar_resources, spawn_sidebar_resources};
use crate::gameplay::sidebar::systems::{spawn_sidebar, update_sidebar_systems, update_status_fps};
use bevy::prelude::{
    in_state, App, FixedUpdate, IntoSystemConfigs, OnEnter, OnExit, Plugin, Update,
};

pub(crate) struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            (spawn_sidebar_resources, spawn_sidebar).chain(),
        );

        app.add_systems(
            Update,
            update_sidebar_systems().run_if(in_state(ApplicationState::Gameplay)),
        );

        app.add_systems(
            FixedUpdate,
            update_status_fps.run_if(in_state(ApplicationState::Gameplay)),
        );

        app.add_systems(
            OnExit(ApplicationState::Gameplay),
            despawn_sidebar_resources,
        );
    }
}
