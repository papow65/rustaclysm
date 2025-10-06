use crate::sidebar::{spawn_sidebar, update_sidebar_systems, update_status_fps};
use application_state::ApplicationState;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::{
    App, FixedUpdate, IntoScheduleConfigs as _, OnEnter, Plugin, Update, in_state,
};

pub(crate) struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }

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
