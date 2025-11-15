use crate::behavior::systems::{behavior_systems, loop_behavior_and_refresh};
use crate::{BehaviorState, behavior::schedule::BehaviorSchedule};
use bevy::prelude::{App, IntoScheduleConfigs as _, Plugin, Res, Update};
use gameplay_resource::GameplayResourcePlugin;

pub(in super::super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(BehaviorSchedule);

        app.add_plugins(GameplayResourcePlugin::<BehaviorState>::default());

        app.add_systems(BehaviorSchedule, behavior_systems());

        app.add_systems(Update, loop_behavior_and_refresh().run_if(looping_behavior));
    }
}

fn looping_behavior(queue: Option<Res<BehaviorState>>) -> bool {
    queue.is_some_and(|queue| queue.looping_behavior())
}
