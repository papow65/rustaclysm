use crate::behavior::schedule::BehaviorSchedule;
use crate::behavior::systems::refresh::refresh_all;
use crate::{BehaviorState, PlayerActionState, RefreshAfterBehavior, RelativeSegments};
use bevy::ecs::schedule::{IntoScheduleConfigs as _, ScheduleConfigs};
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::{State, World, debug, on_message, resource_exists};
use std::time::{Duration, Instant};
use util::log_if_slow;

pub(in super::super) fn loop_behavior_and_refresh() -> ScheduleConfigs<ScheduleSystem> {
    (
        loop_behavior,
        refresh_all().run_if(on_message::<RefreshAfterBehavior>),
    )
        .chain()
        .run_if(resource_exists::<RelativeSegments>)
}

/// This repeatedly runs [`BehaviorSchedule`], until the time runs out or player input is required.
fn loop_behavior(world: &mut World) {
    let start = Instant::now();

    let max_time = if world
        .get_resource::<State<PlayerActionState>>()
        .is_some_and(|state| state.is_still())
    {
        // Allows 10 fps
        Duration::from_secs_f32(0.09)
    } else {
        // Allows 120 fps
        Duration::from_secs_f32(0.005)
    };

    let mut count = 0;
    while looping_behavior(world) {
        world.run_schedule(BehaviorSchedule);
        count += 1;
        if max_time < start.elapsed() {
            debug!("run_behavior_schedule could only handle {count} iterations before the timeout");
            break;
        }
    }

    if 0 < count {
        world.write_message(RefreshAfterBehavior);
    }

    log_if_slow("run_behavior_schedule", start);
}

/// All NPC mave a timeout and the player has an empty instruction queue
fn looping_behavior(world: &World) -> bool {
    world
        .get_resource::<BehaviorState>()
        .expect("BehaviorState should be present")
        .looping_behavior()
}
