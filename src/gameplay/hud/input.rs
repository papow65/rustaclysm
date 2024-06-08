use super::components::ManualDisplay;
use crate::prelude::{log_if_slow, Key, Keys};
use bevy::prelude::{KeyCode, Query, Res, Visibility, With};
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_hud_keyboard_input(
    keys: Res<Keys>,
    mut manual: Query<&mut Visibility, With<ManualDisplay>>,
) {
    let start = Instant::now();

    for _ in keys
        .just_pressed_without_ctrl()
        .filter(|key| **key == Key::Code(KeyCode::F1))
    {
        let mut visibility = manual.single_mut();
        *visibility = if *visibility == Visibility::Hidden {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    log_if_slow("manage_hud_keyboard_input", start);
}
