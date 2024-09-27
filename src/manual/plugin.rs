use crate::manual::input::manage_manual_keyboard_input;
use crate::manual::systems::{spawn_manual, update_manual};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{on_event, App, IntoSystemConfigs, Plugin, PostStartup, Update};

pub(crate) struct ManualPlugin;

impl Plugin for ManualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_manual);

        app.add_systems(
            Update,
            (
                manage_manual_keyboard_input.run_if(on_event::<KeyboardInput>()),
                update_manual, // TODO run_if
            ),
        );
    }
}
