use crate::prelude::Nbor;
use bevy::prelude::Entity;

use super::HorizontalDirection;

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
    /** Redundantly named to avoid confusion */
    MoveItem {
        item: Entity,
        to: Nbor,
    },
    /** Redundantly named to avoid confusion */
    ExamineItem {
        item: Entity,
    },
    ChangePace,
}
