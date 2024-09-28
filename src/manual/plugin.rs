use crate::gameplay::GameplayScreenState;
use crate::manual::input::create_manual_key_bindings;
use crate::manual::systems::{spawn_manual, update_manual};
use bevy::prelude::{
    resource_changed_or_removed, App, IntoSystemConfigs, Plugin, PostStartup, State, Update,
};

pub(crate) struct ManualPlugin;

impl Plugin for ManualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostStartup,
            (
                (spawn_manual, update_manual).chain(),
                create_manual_key_bindings,
            ),
        );

        app.add_systems(
            Update,
            update_manual.run_if(resource_changed_or_removed::<State<GameplayScreenState>>()),
        );
    }
}
