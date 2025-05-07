use crate::systems::{
    load_fonts, manage_button_color, manage_button_input, update_scroll_position,
};
use bevy::prelude::{App, Plugin, Startup, Update};

/// Plugin for the all generic HUD infrastructure
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_fonts);

        app.add_systems(
            Update,
            (
                update_scroll_position,
                manage_button_color,
                manage_button_input::<()>,
            ),
        );
    }
}
