use bevy::ecs::schedule::ScheduleLabel;

#[derive(Clone, Debug, PartialEq, Eq, Hash, ScheduleLabel)]
pub(super) struct KeyComboSchedule;
