use crate::prelude::{ApplicationState, Ctrl, GameplayScreenState, InputChange, Key, Keys};
use bevy::prelude::{KeyCode, NextState, ResMut};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_gameplay_keyboard_input(
    mut keys: Keys,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
) {
    for combo in keys
        .combos(Ctrl::Without)
        .filter(|combo| combo.change == InputChange::JustPressed)
    {
        if let Key::Code(KeyCode::F12) = combo.key {
            next_gameplay_state.set(GameplayScreenState::Inapplicable);
            next_application_state.set(ApplicationState::MainMenu);
        }
    }
}
