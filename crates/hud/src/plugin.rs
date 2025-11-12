use crate::systems::{
    load_fonts, manage_button_color, manage_button_input, toggle_scroll_bar,
    update_scroll_position, update_scroll_thumb,
};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::{
    App, FixedUpdate, IntoScheduleConfigs as _, Plugin, Startup, Update, on_message,
};
use bevy::ui_widgets::ScrollbarPlugin;

/// Plugin for the all generic HUD infrastructure
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScrollbarPlugin);

        app.add_systems(Startup, load_fonts);

        app.add_systems(
            Update,
            (
                update_scroll_position.run_if(on_message::<MouseWheel>),
                update_scroll_thumb,
                manage_button_color,
                manage_button_input::<()>,
            ),
        );

        app.add_systems(FixedUpdate, toggle_scroll_bar);
    }
}
