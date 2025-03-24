use crate::keyboard::{Ctrl, Held, key_binding::KeyBinding, keys::Keys};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{ButtonInput, Commands, Entity, EventReader, In, KeyCode, Query, Res};

#[expect(clippy::needless_pass_by_value)]
pub(super) fn preprocess_keyboard_input(
    mut keyboard_inputs: EventReader<KeyboardInput>,
    key_states: Res<ButtonInput<KeyCode>>,
) -> Keys {
    Keys::new(&mut keyboard_inputs, &key_states)
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_binded_keyboard_input(
    In(keys): In<Keys>,
    mut commands: Commands,
    key_bindings_fresh_without_ctrl: Query<(Entity, &KeyBinding<(), ()>)>,
    key_bindings_held_without_ctrl: Query<(Entity, &KeyBinding<(), Held>)>,
    key_bindings_fresh_with_ctrl: Query<(Entity, &KeyBinding<Ctrl, ()>)>,
    key_bindings_held_with_ctrl: Query<(Entity, &KeyBinding<Ctrl, Held>)>,
) {
    keys.process(
        &mut commands,
        &key_bindings_fresh_without_ctrl,
        &key_bindings_held_without_ctrl,
        &key_bindings_fresh_with_ctrl,
        &key_bindings_held_with_ctrl,
    );
}
