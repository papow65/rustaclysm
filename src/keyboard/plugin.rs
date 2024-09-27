use crate::keyboard::systems::{manage_global_keyboard_input, preprocess_keyboard_input};
use crate::keyboard::Keys;
use bevy::input::{keyboard::KeyboardInput, InputSystem};
use bevy::prelude::{on_event, App, IntoSystemConfigs, Plugin, PreUpdate, Update};

pub(crate) struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Keys::default());

        app.add_systems(PreUpdate, preprocess_keyboard_input.after(InputSystem));
        app.add_systems(
            Update,
            manage_global_keyboard_input.run_if(on_event::<KeyboardInput>()),
        );
    }
}
