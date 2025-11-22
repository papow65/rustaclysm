use crate::GameplayScreenState;
use crate::screens::tool::systems::{
    adapt_to_tool_deselection, adapt_to_tool_selection, create_tool_key_bindings, spawn_tool_screen,
};
use bevy::prelude::{App, IntoScheduleConfigs as _, OnEnter, Plugin, Update, in_state};
use selection_list::selection_list_plugin;

pub(crate) struct ToolScreenPlugin;

impl Plugin for ToolScreenPlugin {
    fn build(&self, app: &mut App) {
        selection_list_plugin::<_, ()>(app, GameplayScreenState::Tool, "select action");

        app.add_systems(
            OnEnter(GameplayScreenState::Tool),
            (spawn_tool_screen, create_tool_key_bindings),
        );

        app.add_systems(
            Update,
            (adapt_to_tool_selection, adapt_to_tool_deselection)
                .run_if(in_state(GameplayScreenState::Tool)),
        );
    }
}
