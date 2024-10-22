use crate::application::ApplicationState;
use crate::gameplay::{GameplayScreenState, Infos, RelativeSegments};
use bevy::prelude::{Camera2d, Commands, NextState, Res, ResMut, StateScoped};

pub(super) fn spawn_pre_gameplay_camera(mut commands: Commands) {
    commands.spawn((Camera2d, StateScoped(ApplicationState::PreGameplay)));
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn start_gameplay(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_screen_state: ResMut<NextState<GameplayScreenState>>,
    infos: Option<Res<Infos>>,
    relative_segments: Option<Res<RelativeSegments>>,
) {
    if infos.is_some() && relative_segments.is_some() {
        next_application_state.set(ApplicationState::Gameplay);
        next_gameplay_screen_state.set(GameplayScreenState::Loading);
    }
}
