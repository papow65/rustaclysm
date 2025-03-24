use crate::gameplay::focus::systems::{update_camera_base, update_focus_cursor_visibility};
use crate::gameplay::{FocusState, systems::update_visibility};
use bevy::prelude::{
    App, AppExtStates as _, IntoSystemConfigs as _, Plugin, State, Update, in_state, not,
    resource_exists_and_changed,
};
use util::log_transition_plugin;

pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<FocusState>();
        app.add_plugins(log_transition_plugin::<FocusState>);

        app.add_systems(
            Update,
            (
                update_focus_cursor_visibility,
                update_visibility.run_if(not(in_state(FocusState::Normal))),
                update_camera_base,
            )
                .run_if(resource_exists_and_changed::<State<FocusState>>),
        );
    }
}
