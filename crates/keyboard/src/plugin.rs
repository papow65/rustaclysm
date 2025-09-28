use crate::systems::{manage_binded_keyboard_input, preprocess_keyboard_input};
use bevy::input::{InputSystems, keyboard::KeyboardInput};
use bevy::prelude::{
    App, IntoScheduleConfigs as _, IntoSystem as _, Plugin, PreUpdate, on_message,
};

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            preprocess_keyboard_input
                .pipe(manage_binded_keyboard_input)
                .after(InputSystems)
                .run_if(on_message::<KeyboardInput>),
        );
    }
}
