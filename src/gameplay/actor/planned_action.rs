use crate::prelude::{CardinalDirection, HorizontalDirection, Nbor};
use bevy::prelude::Entity;

#[derive(Copy, Clone, Debug)]
pub(crate) enum StayDuration {
    Short,
    Long,
}

#[derive(Debug)]
pub(crate) enum PlannedAction {
    Stay {
        duration: StayDuration,
    },
    Step {
        to: Nbor,
    },
    Attack {
        target: Nbor,
    },
    Smash {
        target: Nbor,
    },
    Pulp {
        target: HorizontalDirection,
    },
    Peek {
        target: CardinalDirection,
    },
    Close {
        target: HorizontalDirection,
    },
    Wield {
        item: Entity,
    },
    Unwield {
        item: Entity,
    },
    Pickup {
        item: Entity,
    },
    /// Redundantly named to avoid confusion
    MoveItem {
        item: Entity,
        to: Nbor,
    },
    /// Redundantly named to avoid confusion
    ExamineItem {
        item: Entity,
    },
    ChangePace,
}
