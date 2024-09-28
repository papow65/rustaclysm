use crate::application::ApplicationState;
use crate::common::log_if_slow;
use crate::keyboard::{Key, KeyBinding};
use bevy::prelude::{In, KeyCode, NextState, ResMut, World};
use std::time::Instant;

pub(crate) fn create_gameplay_key_bindings(world: &mut World) {
    let start = Instant::now();

    let to_main_menu = world.register_system(to_main_menu);
    world.spawn(KeyBinding::from(KeyCode::F12, to_main_menu).scoped(ApplicationState::Gameplay));
    log_if_slow("create_gameplay_key_bindings", start);
}

fn to_main_menu(In(_): In<Key>, mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}
