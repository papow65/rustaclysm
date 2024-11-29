use crate::keyboard::systems::{
    create_global_key_bindings, manage_binded_keyboard_input, preprocess_keyboard_input,
};
use bevy::input::{keyboard::KeyboardInput, InputSystem};
use bevy::prelude::{
    on_event, App, IntoSystem as _, IntoSystemConfigs as _, Plugin, PreUpdate, Startup,
};

pub(crate) struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_global_key_bindings);

        app.add_systems(
            PreUpdate,
            preprocess_keyboard_input
                .pipe(manage_binded_keyboard_input)
                .after(InputSystem)
                .run_if(on_event::<KeyboardInput>),
        );
    }
}
