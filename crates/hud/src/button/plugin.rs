use crate::button::systems::{manage_button_color, manage_button_input};
use bevy::prelude::{App, Plugin, Update};

pub(crate) struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (manage_button_color, manage_button_input::<()>));
    }
}
