use crate::{PlayerActionState, behavior::BehaviorPlugin};
use bevy::prelude::{App, AppExtStates as _, Plugin};
use util::log_transition_plugin;

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehaviorPlugin);

        app.add_sub_state::<PlayerActionState>();
        app.add_plugins(log_transition_plugin::<PlayerActionState>);
    }
}
