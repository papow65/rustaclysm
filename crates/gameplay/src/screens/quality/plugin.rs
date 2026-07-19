use crate::screens::quality::systems::{create_quality_screen_key_bindings, spawn_quality_screen};
use bevy::prelude::{App, OnEnter, Plugin};
use gameplay_screen_state::GameplayScreenState;

pub(crate) struct QualityScreenPlugin;

impl Plugin for QualityScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Quality),
            (spawn_quality_screen, create_quality_screen_key_bindings),
        );
    }
}
