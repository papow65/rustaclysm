use application_state::ApplicationState;
use bevy::prelude::{
    App, IntoScheduleConfigs as _, NextState, Plugin, ResMut, Update, debug, in_state,
};
use gameplay_transition_state::GameplayTransitionState;

pub(crate) struct UnloadingScreenPlugin;

impl Plugin for UnloadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            finish_unloading.run_if(in_state(GameplayTransitionState::Unloading)),
        );
    }
}

fn finish_unloading(mut next_application_state: ResMut<NextState<ApplicationState>>) {
    debug!("Unloading complete");
    next_application_state.set(ApplicationState::MainMenu);
}
