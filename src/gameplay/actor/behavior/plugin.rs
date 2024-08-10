use crate::common::log_transition_plugin;
use crate::gameplay::actor::behavior::schedule::BehaviorSchedule;
use crate::gameplay::actor::behavior::state::BehaviorState;
use crate::gameplay::actor::behavior::system_configs::{
    behavior_systems, loop_behavior_and_refresh,
};
use bevy::prelude::{in_state, App, AppExtStates, IntoSystemConfigs, Plugin, Update};

pub(in super::super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<BehaviorState>();
        app.add_plugins(log_transition_plugin::<BehaviorState>);

        app.add_systems(
            Update,
            loop_behavior_and_refresh().run_if(in_state(BehaviorState)),
        );

        app.init_schedule(BehaviorSchedule);

        app.add_systems(BehaviorSchedule, behavior_systems());
    }
}
