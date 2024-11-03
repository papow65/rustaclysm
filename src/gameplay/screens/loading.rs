use crate::gameplay::{GameplayReadiness, GameplayScreenState};
use bevy::prelude::{in_state, App, IntoSystemConfigs, NextState, Plugin, ResMut, Update};

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
pub(crate) fn finish_loading(
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    gameplay_readiness: GameplayReadiness,
) {
    if gameplay_readiness.ready_to_run() {
        println!("Loading complete");
        next_gameplay_state.set(GameplayScreenState::Base);
    }
}
