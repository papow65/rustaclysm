use crate::systems::{manage_binded_keyboard_input, preprocess_keyboard_input};
use bevy::input::{InputSystem, keyboard::KeyboardInput};
use bevy::prelude::{App, IntoSystem as _, IntoSystemConfigs as _, Plugin, PreUpdate, on_event};

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            preprocess_keyboard_input
                .pipe(manage_binded_keyboard_input)
                .after(InputSystem)
                .run_if(on_event::<KeyboardInput>),
        );
    }
}
