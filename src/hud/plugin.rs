use crate::hud::systems::{
    load_fonts, manage_button_color, manage_button_input, manage_scroll_lists, resize_scroll_lists,
};
use bevy::prelude::{
    App, Condition as _, IntoSystemConfigs as _, Plugin, Startup, UiScale, Update, on_event,
    resource_exists_and_changed,
};
use bevy::{input::mouse::MouseWheel, window::WindowResized};

/// Plugin for the all generic HUD infrastructure
pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_fonts);

        app.add_systems(
            Update,
            (
                manage_button_color,
                manage_button_input::<()>,
                manage_scroll_lists.run_if(on_event::<MouseWheel>),
                resize_scroll_lists
                    .run_if(on_event::<WindowResized>.or(resource_exists_and_changed::<UiScale>)),
            ),
        );
    }
}
