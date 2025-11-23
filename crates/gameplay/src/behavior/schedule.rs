use bevy::ecs::schedule::ScheduleLabel;

/// This schedule attempts to execute one character action.
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub(super) struct BehaviorSchedule;
