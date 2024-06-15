use super::{
    schedule::BehaviorSchedule, system_configs::behavior_systems, systems::refresh::trigger_refresh,
};
use crate::prelude::{CardinalDirection, PlayerActionState};
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
        let peeking = PlayerActionState::Peeking {
            active_target: Some(direction),
        };
        app.add_systems(OnEnter(peeking.clone()), trigger_refresh);
        app.add_systems(OnExit(peeking), trigger_refresh);
    }
}
