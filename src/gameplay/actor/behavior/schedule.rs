use crate::prelude::{log_if_slow, InstructionQueue, PlayerActionState, RefreshAfterBehavior};
use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemState},
    prelude::{Res, State, World},
};
use std::time::{Duration, Instant};

/** This schedule attempts to execute one character action. */
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub(super) struct BehaviorSchedule;

impl BehaviorSchedule {
    /** This repeatedly runs [`BehaviorSchedule`], until the time runs out or player input is required. */
    pub(super) fn repeat(world: &mut World) {
        let start = Instant::now();

        let max_time = if world
            .get_resource::<State<PlayerActionState>>()
            .is_some_and(|state| {
                matches!(
                    **state,
                    PlayerActionState::Waiting { .. } | PlayerActionState::Sleeping { .. }
                )
            }) {
            // Allows 10 fps
            Duration::from_secs_f32(0.09)
        } else {
            // Allows 120 fps
            Duration::from_secs_f32(0.005)
        };

        let mut count = 0;
        while !Self::waiting_for_user_input(world) {
            world.run_schedule(Self);
            count += 1;
            if max_time < start.elapsed() {
                println!(
                    "run_behavior_schedule could only handle {count} iterations before the timeout"
                );
                break;
            }
        }

        if 0 < count {
            world.send_event(RefreshAfterBehavior);
        }

        log_if_slow("run_behavior_schedule", start);
    }

    /** All NPC mave a timeout and the player has an empty instruction queue */
    fn waiting_for_user_input(world: &mut World) -> bool {
        let mut system_state = SystemState::<(Res<InstructionQueue>,)>::new(world);
        let (instruction_queue,) = system_state.get(world);
        instruction_queue.is_waiting()
    }
}
