use crate::{button::ButtonPlugin, panel::PanelPlugin, text::TextPlugin};
use bevy::prelude::{App, Plugin};

/// Plugin for the all generic HUD infrastructure
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ButtonPlugin, TextPlugin, PanelPlugin));
    }
}
