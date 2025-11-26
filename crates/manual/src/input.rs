use crate::{ManualSection, components::ManualDisplay};
use bevy::prelude::{KeyCode, Query, Visibility, With, World, error};
use keyboard::KeyBindings;
use std::time::Instant;
use util::log_if_slow;

pub(super) fn create_manual_key_bindings(world: &mut World) {
    KeyBindings::<_, (), ()>::spawn_global(world, |bindings| {
        bindings.add(KeyCode::F1, toggle_manual);
        bindings.add('?', toggle_manual);
    });

    world.spawn(ManualSection::new(&[("key bindings", "F1/?")], u8::MAX - 1));
}

fn toggle_manual(mut manual: Query<&mut Visibility, With<ManualDisplay>>) {
    let start = Instant::now();

    let Ok(mut visibility) = manual.single_mut().inspect_err(|error| error!("{error:?}")) else {
        return;
    };
    *visibility = if *visibility == Visibility::Hidden {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    log_if_slow("toggle_manual", start);
}
