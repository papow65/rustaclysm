use crate::gameplay::{CardinalDirection, ChangePace, HorizontalDirection, Nbor, StartCraft};
use bevy::prelude::Entity;

#[derive(Debug)]
pub(crate) enum PlannedAction {
    Stay,
    Sleep,
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
    StartCraft(StartCraft),
    ContinueCraft {
        item: Entity,
    },
    /// Redundantly named to avoid confusion
    ExamineItem {
        item: Entity,
    },
    ChangePace(ChangePace),
}
