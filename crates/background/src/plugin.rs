use crate::state::BackgroundState;
use crate::systems::{
    load_background, resize_background, spawn_background, spawn_background_camera,
};
use bevy::prelude::{
    App, AppExtStates as _, FixedUpdate, IntoScheduleConfigs as _, OnEnter, Plugin, Startup,
    SystemCondition as _, Update, on_message, state_exists,
};
use bevy::window::{RequestRedraw, WindowResized};
use util::log_transition_plugin;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<BackgroundState>();
        app.add_plugins(log_transition_plugin::<BackgroundState>);

        app.add_systems(Startup, load_background);

        app.add_systems(
            OnEnter(BackgroundState),
            (spawn_background_camera, spawn_background),
        );

        app.add_systems(
            Update,
            resize_background.run_if(
                (on_message::<WindowResized>.or(on_message::<RequestRedraw>))
                    .and(state_exists::<BackgroundState>),
            ),
        );

        // WindowResized is not triggered when maximizing the window using Bevy 0.14.1, so we need this fallback
        app.add_systems(FixedUpdate, resize_background);
    }
}
