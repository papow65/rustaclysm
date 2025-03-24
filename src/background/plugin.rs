use crate::background::state::BackgroundState;
use crate::background::systems::{resize_background, spawn_background};
use bevy::prelude::{
    App, AppExtStates as _, Condition as _, FixedUpdate, IntoSystemConfigs as _, OnEnter, Plugin,
    Update, on_event, state_exists,
};
use bevy::window::{RequestRedraw, WindowResized};
use util::log_transition_plugin;

pub(crate) struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<BackgroundState>();
        app.enable_state_scoped_entities::<BackgroundState>();
        app.add_plugins(log_transition_plugin::<BackgroundState>);

        app.add_systems(OnEnter(BackgroundState), spawn_background);

        app.add_systems(
            Update,
            resize_background.run_if(
                (on_event::<WindowResized>.or(on_event::<RequestRedraw>))
                    .and(state_exists::<BackgroundState>),
            ),
        );

        // WindowResized is not triggered when maximizing the window using Bevy 0.14.1, so we need this fallback
        app.add_systems(FixedUpdate, resize_background);
    }
}
