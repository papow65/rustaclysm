use crate::application::ApplicationState;
use crate::keyboard::{Key, Keys};
use bevy::prelude::{KeyCode, NextState, Res, ResMut};

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn manage_gameplay_keyboard_input(
    keys: Res<Keys>,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
) {
    for _ in keys
        .just_pressed_without_ctrl()
        .filter(|key| **key == Key::Code(KeyCode::F12))
    {
        next_application_state.set(ApplicationState::MainMenu);
    }
}
