use crate::{GameplayReadiness, GameplayScreenState};
use bevy::prelude::{
    App, IntoScheduleConfigs as _, NextState, Plugin, ResMut, Update, debug, in_state,
};

pub(crate) struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            finish_loading.run_if(in_state(GameplayScreenState::Loading)),
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn finish_loading(
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    gameplay_readiness: GameplayReadiness,
) {
    if gameplay_readiness.ready_to_run() {
        debug!("Loading complete");
        next_gameplay_state.set(GameplayScreenState::Base);
    }
}
