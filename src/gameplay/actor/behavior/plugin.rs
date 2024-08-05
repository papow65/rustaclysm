use crate::gameplay::actor::behavior::schedule::BehaviorSchedule;
use crate::gameplay::actor::behavior::system_configs::behavior_systems;
use crate::gameplay::actor::behavior::systems::refresh::trigger_refresh;
use crate::gameplay::{CardinalDirection, PlayerActionState};
use bevy::prelude::{App, OnEnter, OnExit, Plugin};

pub(in super::super) struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(BehaviorSchedule);

        app.add_systems(BehaviorSchedule, behavior_systems());

        trigger_peeking_refresh(app);
    }
}

fn trigger_peeking_refresh(app: &mut App) {
    for direction in CardinalDirection::ALL {
        let peeking = PlayerActionState::Peeking { direction };
        app.add_systems(OnEnter(peeking.clone()), trigger_refresh);
        app.add_systems(OnExit(peeking), trigger_refresh);
    }
}
