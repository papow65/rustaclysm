use crate::application::ApplicationState;
use crate::gameplay::sidebar::systems::{spawn_sidebar, update_sidebar_systems, update_status_fps};
use bevy::prelude::{in_state, App, FixedUpdate, IntoSystemConfigs as _, OnEnter, Plugin, Update};

pub(crate) struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ApplicationState::Gameplay), spawn_sidebar);

        app.add_systems(
            Update,
            update_sidebar_systems().run_if(in_state(ApplicationState::Gameplay)),
        );

        app.add_systems(
            FixedUpdate,
            update_status_fps.run_if(in_state(ApplicationState::Gameplay)),
        );
    }
}
