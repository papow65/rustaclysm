use bevy::ecs::schedule::ScheduleLabel;

/** This is only run when the game when any character acts, sometimes multiple times per tick. */
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub(super) struct BehaviorSchedule;
