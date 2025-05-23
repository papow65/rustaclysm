use crate::behavior::systems::{behavior_systems, loop_behavior_and_refresh};
use crate::behavior::{schedule::BehaviorSchedule, state::BehaviorState};
use bevy::prelude::{App, AppExtStates as _, IntoScheduleConfigs as _, Plugin, Update, in_state};
use util::log_transition_plugin;

pub(in super::super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<BehaviorState>();
        app.add_plugins(log_transition_plugin::<BehaviorState>);

        app.init_schedule(BehaviorSchedule);
        app.add_systems(BehaviorSchedule, behavior_systems());

        app.add_systems(
            Update,
            loop_behavior_and_refresh().run_if(in_state(BehaviorState)),
        );
    }
}
