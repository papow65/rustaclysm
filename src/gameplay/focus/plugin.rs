use crate::gameplay::focus::systems::{update_camera_base, update_focus_cursor_visibility};
use crate::gameplay::{loop_behavior_and_refresh, update_visibility, FocusState};
use bevy::prelude::{
    in_state, not, resource_exists_and_changed, App, AppExtStates, IntoSystemConfigs, Plugin,
    State, Update,
};

pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<FocusState>();

        app.add_systems(
            Update,
            (
                loop_behavior_and_refresh().run_if(in_state(FocusState::Normal)),
                (
                    update_focus_cursor_visibility,
                    update_visibility.run_if(not(in_state(FocusState::Normal))),
                    update_camera_base,
                )
                    .run_if(resource_exists_and_changed::<State<FocusState>>),
            ),
        );
    }
}
