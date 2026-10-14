use crate::button::systems::{
    manage_button_input, manage_interacted_button_color, manage_new_button,
};
use bevy::prelude::{App, Plugin, Update};

pub(crate) struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_new_button,
                manage_interacted_button_color,
                manage_button_input::<()>,
            ),
        );
    }
}
