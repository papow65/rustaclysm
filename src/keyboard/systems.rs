use crate::keyboard::{key_binding::KeyBinding, keys::Keys, Ctrl, Held, Key, KeyBindings};
use crate::manual::ManualSection;
use bevy::app::AppExit;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{
    ButtonInput, Commands, EventReader, Events, In, KeyCode, Query, Res, ResMut, UiScale, World,
};

pub(super) fn create_global_key_bindings(world: &mut World) {
    KeyBindings::<_, Ctrl, ()>::spawn_global(
        world,
        |bindings| {
            bindings.add_multi(['+', '-'], zoom_ui);
            bindings.add_multi(['c', 'q'], quit);
        },
        ManualSection::new(&[("zoom ui", "ctrl +/-"), ("quit", "ctrl c/q")], u8::MAX),
    );
}

fn zoom_ui(In(key): In<Key>, mut ui_scale: ResMut<UiScale>) {
    let zoom = match key {
        Key::Character('+') => 1,
        Key::Character('-') => -1,
        _ => panic!("Unexpected key {key:?}"),
    };

    let px = zoom + (16.0 * ui_scale.0) as i8;
    let px = px.clamp(4, 64);
    ui_scale.0 = f32::from(px) / 16.0;
    println!("UI scale: {ui_scale:?}");
}

fn quit(In(_): In<Key>, mut app_exit_events: ResMut<Events<AppExit>>) {
    app_exit_events.send(AppExit::Success);
}

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
    key_bindings_fresh_without_ctrl: Query<&KeyBinding<(), ()>>,
    key_bindings_held_without_ctrl: Query<&KeyBinding<(), Held>>,
    key_bindings_fresh_with_ctrl: Query<&KeyBinding<Ctrl, ()>>,
    key_bindings_held_with_ctrl: Query<&KeyBinding<Ctrl, Held>>,
) {
    keys.process(
        &mut commands,
        &key_bindings_fresh_without_ctrl,
        &key_bindings_held_without_ctrl,
        &key_bindings_fresh_with_ctrl,
        &key_bindings_held_with_ctrl,
    );
}
