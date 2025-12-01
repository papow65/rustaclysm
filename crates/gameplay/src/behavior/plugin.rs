use crate::behavior::systems::{behavior_systems, loop_behavior_and_refresh};
use crate::{BehaviorValidator, PlayerInstructions, behavior::schedule::BehaviorSchedule};
use bevy::prelude::{App, IntoScheduleConfigs as _, Plugin, Update};
use gameplay_resource::gameplay_resource_plugin;
use util::log_resource_change_plugin;

pub(in super::super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(BehaviorSchedule);

        app.add_plugins(gameplay_resource_plugin::<PlayerInstructions>);
        app.add_plugins(log_resource_change_plugin::<PlayerInstructions>);

        app.add_systems(BehaviorSchedule, behavior_systems());

        app.add_systems(Update, loop_behavior_and_refresh().run_if(looping_behavior));
    }
}

#[expect(clippy::needless_pass_by_value)]
fn looping_behavior(behavior_validator: BehaviorValidator) -> bool {
    behavior_validator.looping_behavior()
}
