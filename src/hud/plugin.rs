use crate::hud::systems::{
    create_default_panel, load_fonts, manage_button_color, manage_button_input,
    manage_scrolling_lists, resize_scrolling_lists,
};
use bevy::prelude::{
    on_event, resource_exists_and_changed, App, Condition, IntoSystemConfigs, Plugin, Startup,
    UiScale, Update,
};
use bevy::{input::mouse::MouseWheel, window::WindowResized};

/// Plugin for the all generic HUD infrastructure
pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (create_default_panel, load_fonts));

        app.add_systems(
            Update,
            (
                manage_button_color,
                manage_button_input::<()>,
                manage_scrolling_lists.run_if(on_event::<MouseWheel>()),
                resize_scrolling_lists.run_if(
                    on_event::<WindowResized>().or_else(resource_exists_and_changed::<UiScale>),
                ),
            ),
        );
    }
}
