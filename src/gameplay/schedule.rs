use crate::prelude::*;
use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemState},
    prelude::*,
};
use std::time::{Duration, Instant};

// A label for our new Schedule!
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct BehaviorSchedule;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum UpdateSet {
    ManageBehavior,
    FlushBehavior,
    ApplyEffects,
    FlushEffects,
}

pub(crate) fn behavior_schedule() -> Schedule {
    let mut behavior_schedule = Schedule::new();
    behavior_schedule.set_default_base_set(CoreSet::Update);

    behavior_schedule.add_systems(
        (plan_action.pipe(perform_action).pipe(handle_impact),)
            .in_set(UpdateSet::ManageBehavior)
            .in_set(OnUpdate(ApplicationState::Gameplay)),
    );
    behavior_schedule.add_systems(
        (apply_system_buffers,)
            .in_set(UpdateSet::FlushBehavior)
            .after(UpdateSet::ManageBehavior)
            .in_set(OnUpdate(ApplicationState::Gameplay)),
    );
    behavior_schedule.add_systems(
        (
            manage_player_death,
            toggle_doors,
            update_damaged_characters,
            update_damaged_items,
        )
            .in_set(UpdateSet::ApplyEffects)
            .after(UpdateSet::FlushBehavior)
            .in_set(OnUpdate(ApplicationState::Gameplay)),
    );
    behavior_schedule.add_systems(
        (apply_system_buffers,)
            .in_set(UpdateSet::FlushEffects)
            .after(UpdateSet::ApplyEffects)
            .in_set(OnUpdate(ApplicationState::Gameplay)),
    );
    behavior_schedule
}

pub(crate) fn run_behavior_schedule(world: &mut World) {
    let start = Instant::now();

    let mut count = 0;
    while !waiting_for_user_input(world) && !over_time(&start, count) {
        world.run_schedule(BehaviorSchedule);
        count += 1;
    }

    log_if_slow("run_behavior_schedule", start);
}

/** All NPC mave a timeout and the player has an empty instruction queue */
fn waiting_for_user_input(world: &mut World) -> bool {
    let mut system_state = SystemState::<(Res<InstructionQueue>,)>::new(world);
    let (instruction_queue,) = system_state.get(world);
    instruction_queue.is_waiting()
}

fn over_time(start: &Instant, count: usize) -> bool {
    let over_time = Duration::from_millis(2) * 3 / 4 < start.elapsed();
    if over_time {
        eprintln!("run_behavior_schedule could ony handle {count} iterations before the timeout");
    }
    over_time
}
