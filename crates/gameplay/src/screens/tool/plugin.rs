use crate::GameplayScreenState;
use crate::screens::tool::systems::{create_tool_key_bindings, spawn_tool_screen};
use bevy::prelude::{App, OnEnter, Plugin};

pub(crate) struct ToolScreenPlugin;

impl Plugin for ToolScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameplayScreenState::Tool),
            (spawn_tool_screen, create_tool_key_bindings),
        );
    }
}
