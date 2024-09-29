use crate::common::log_if_slow;
use crate::keyboard::{Key, KeyBindings};
use crate::manual::{components::ManualDisplay, ManualSection};
use bevy::prelude::{In, KeyCode, Query, Visibility, With, World};
use std::time::Instant;

pub(super) fn create_manual_key_bindings(world: &mut World) {
    KeyBindings::<_, (), ()>::spawn_global(
        world,
        |bindings| {
            bindings.add(KeyCode::F1, toggle_manual);
        },
        ManualSection::new(&[("toggle this", "F1")], u8::MAX - 1),
    );
}

fn toggle_manual(In(_): In<Key>, mut manual: Query<&mut Visibility, With<ManualDisplay>>) {
    let start = Instant::now();

    let mut visibility = manual.single_mut();
    *visibility = if *visibility == Visibility::Hidden {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    log_if_slow("manage_manual_keyboard_input", start);
}
