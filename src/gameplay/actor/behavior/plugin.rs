use super::{schedule::BehaviorSchedule, system_configs::behavior_systems};
use bevy::prelude::{App, Plugin};

pub(in super::super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(BehaviorSchedule);

        app.add_systems(BehaviorSchedule, behavior_systems());
    }
}
