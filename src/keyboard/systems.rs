use crate::keyboard::{Key, Keys};
use bevy::app::AppExit;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{ButtonInput, EventReader, Events, KeyCode, Res, ResMut, UiScale};

#[expect(clippy::needless_pass_by_value)]
pub(super) fn preprocess_keyboard_input(
    mut keyboard_inputs: EventReader<KeyboardInput>,
    key_states: Res<ButtonInput<KeyCode>>,
    mut keys: ResMut<Keys>,
) {
    keys.update(&mut keyboard_inputs, &key_states);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_global_keyboard_input(
    keys: Res<Keys>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut ui_scale: ResMut<UiScale>,
) {
    for key_change in keys.with_ctrl() {
        match key_change.key {
            Key::Character('c' | 'q') => {
                app_exit_events.send(AppExit::Success);
            }
            Key::Character(resize @ ('+' | '-')) => {
                let px = if resize == '+' { 1 } else { -1 } + (16.0 * ui_scale.0) as i8;
                let px = px.clamp(4, 64);
                ui_scale.0 = f32::from(px) / 16.0;
                println!("UI scale: {ui_scale:?}");
            }
            _ => {}
        }
    }
}
