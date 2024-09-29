use crate::application::ApplicationState;
use crate::common::log_if_slow;
use crate::keyboard::{Key, KeyBindings};
use crate::manual::ManualSection;
use bevy::prelude::{In, KeyCode, Local, NextState, ResMut, World};
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_gameplay_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<ApplicationState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(
        world,
        ApplicationState::Gameplay,
        |bindings| {
            bindings.add(KeyCode::F12, to_main_menu);
        },
        ManualSection::new(&[("to main menu", "F12")], u8::MAX - 2),
    );

    log_if_slow("create_gameplay_key_bindings", start);
}

fn to_main_menu(In(_): In<Key>, mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}
