use crate::panel::systems::{
    add_node_outlines, toggle_scroll_bar_visibility, update_scroll_position,
    update_scroll_thumb_color,
};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::{App, FixedUpdate, IntoScheduleConfigs as _, Plugin, Update, on_message};
use bevy::ui_widgets::ScrollbarPlugin;
use std::env;

pub(crate) struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScrollbarPlugin);

        app.add_systems(
            Update,
            (
                update_scroll_position.run_if(on_message::<MouseWheel>),
                update_scroll_thumb_color,
            ),
        );

        app.add_systems(FixedUpdate, toggle_scroll_bar_visibility);

        if env::var("UI_OUTLINES") == Ok(String::from("1")) {
            app.add_observer(add_node_outlines);
        }
    }
}
