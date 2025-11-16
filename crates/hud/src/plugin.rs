use crate::{Fonts, button::ButtonPlugin, panel::PanelPlugin, selection_list::SelectionListPlugin};
use bevy::prelude::{App, Plugin};

/// Plugin for the all generic HUD infrastructure
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ButtonPlugin, PanelPlugin, SelectionListPlugin));

        app.init_resource::<Fonts>();
    }
}
