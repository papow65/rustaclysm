use crate::common::log_transition_plugin;
use crate::gameplay::{behavior::BehaviorPlugin, PlayerActionState};
use bevy::prelude::{App, AppExtStates, Plugin};

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehaviorPlugin);

        app.add_sub_state::<PlayerActionState>();
        app.add_plugins((log_transition_plugin::<PlayerActionState>,));
    }
}
