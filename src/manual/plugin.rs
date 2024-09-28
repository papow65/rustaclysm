use crate::manual::input::create_manual_key_bindings;
use crate::manual::systems::{spawn_manual, update_manual};
use bevy::prelude::{App, Plugin, PostStartup, Update};

pub(crate) struct ManualPlugin;

impl Plugin for ManualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn_manual, create_manual_key_bindings));

        app.add_systems(
            Update,
            update_manual, // TODO run_if
        );
    }
}
