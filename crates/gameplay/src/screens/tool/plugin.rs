use crate::GameplayScreenState;
use crate::screens::tool::systems::{
    adapt_to_tool_selection, create_tool_key_bindings, spawn_tool_screen,
};
use bevy::prelude::{App, OnEnter, Plugin, With};
use selection_list::{SelectionList, selection_list_plugin};

pub(crate) struct ToolScreenPlugin;

impl Plugin for ToolScreenPlugin {
    fn build(&self, app: &mut App) {
        selection_list_plugin::<_, _, _, With<SelectionList>>(
            app,
            GameplayScreenState::Tool,
            "select action",
            adapt_to_tool_selection,
        );

        app.add_systems(
            OnEnter(GameplayScreenState::Tool),
            (spawn_tool_screen, create_tool_key_bindings),
        );
    }
}
