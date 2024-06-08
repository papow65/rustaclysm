use crate::prelude::{ApplicationState, GameplayScreenState, Key, Keys};
use bevy::prelude::{KeyCode, NextState, Res, ResMut};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_gameplay_keyboard_input(
    keys: Res<Keys>,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
) {
    for _ in keys
        .just_pressed_without_ctrl()
        .filter(|key| **key == Key::Code(KeyCode::F12))
    {
        next_gameplay_state.set(GameplayScreenState::Inapplicable);
        next_application_state.set(ApplicationState::MainMenu);
    }
}
