use application_state::ApplicationState;
use bevy::prelude::{Camera2d, Commands, NextState, ResMut, StateScoped};
use gameplay::{GameplayReadiness, GameplayScreenState};

pub(super) fn spawn_pre_gameplay_camera(mut commands: Commands) {
    commands.spawn((Camera2d, StateScoped(ApplicationState::PreGameplay)));
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn start_gameplay(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_screen_state: ResMut<NextState<GameplayScreenState>>,
    gameplay_readiness: GameplayReadiness,
) {
    if gameplay_readiness.ready_to_load() {
        next_application_state.set(ApplicationState::Gameplay);
        next_gameplay_screen_state.set(GameplayScreenState::Loading);
    }
}
