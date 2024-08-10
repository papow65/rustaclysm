use crate::background::state::BackgroundState;
use crate::background::systems::{resize_background, spawn_background};
use crate::common::log_transition_plugin;
use bevy::prelude::{
    on_event, state_exists, App, AppExtStates, Condition, FixedUpdate, IntoSystemConfigs, OnEnter,
    Plugin, Update,
};
use bevy::window::{RequestRedraw, WindowResized};

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
                (on_event::<WindowResized>().or_else(on_event::<RequestRedraw>()))
                    .and_then(state_exists::<BackgroundState>),
            ),
        );

        // WindowResized is not triggered when maximizing the window using Bevy 0.14.1, so we need this fallback
        app.add_systems(FixedUpdate, resize_background);
    }
}
