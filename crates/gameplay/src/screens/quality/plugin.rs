use crate::GameplayScreenState;
use crate::screens::quality::systems::{create_crafting_key_bindings, spawn_crafting_screen};
use bevy::prelude::{App, OnEnter, Plugin};

pub(crate) struct QualityScreenPlugin;

impl Plugin for QualityScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Quality),
            (spawn_crafting_screen, create_crafting_key_bindings),
        );
    }
}
