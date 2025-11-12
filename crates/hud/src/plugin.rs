use crate::systems::{
    load_fonts, manage_button_color, manage_button_input, update_scroll_position,
    update_scroll_thumb,
};
use bevy::prelude::{App, Plugin, Startup, Update};
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
                update_scroll_position,
                update_scroll_thumb,
                manage_button_color,
                manage_button_input::<()>,
            ),
        );
    }
}
