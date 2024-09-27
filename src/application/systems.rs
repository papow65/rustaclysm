use crate::application::ApplicationState;
use bevy::prelude::{NextState, Query, ResMut, Window};

pub(super) fn maximize_window(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.set_maximized(true);
    }
}

pub(super) fn enter_main_menu(mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}
