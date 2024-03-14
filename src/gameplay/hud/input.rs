use super::ManualDisplay;
use crate::prelude::*;
use bevy::prelude::*;
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_hud_keyboard_input(
    mut keys: Keys,
    mut manual: Query<&mut Visibility, With<ManualDisplay>>,
) {
    let start = Instant::now();

    for _ in keys.combos(Ctrl::Without).filter(|combo| {
        combo.key == Key::Code(KeyCode::F1) && combo.change == InputChange::JustPressed
    }) {
        let mut visibility = manual.single_mut();
        *visibility = if *visibility == Visibility::Hidden {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    log_if_slow("manage_hud_keyboard_input", start);
}
