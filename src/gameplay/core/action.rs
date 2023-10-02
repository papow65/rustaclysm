use crate::prelude::{ActorChange, Nbor};
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
    Close {
        target: Nbor,
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

#[derive(Clone, Debug)]
pub(crate) struct Stay {
    pub(crate) duration: StayDuration,
}

impl ActorChange for Stay {}

#[derive(Clone, Debug)]
pub(crate) struct Step {
    pub(crate) to: Nbor,
}

impl ActorChange for Step {}

#[derive(Clone, Debug)]
pub(crate) struct Attack {
    pub(crate) target: Nbor,
}

impl ActorChange for Attack {}

#[derive(Clone, Debug)]
pub(crate) struct Smash {
    pub(crate) target: Nbor,
}

impl ActorChange for Smash {}

#[derive(Clone, Debug)]
pub(crate) struct Close {
    pub(crate) target: Nbor,
}

impl ActorChange for Close {}

#[derive(Clone, Debug)]
pub(crate) struct Wield {
    pub(crate) item: Entity,
}

impl ActorChange for Wield {}

#[derive(Clone, Debug)]
pub(crate) struct Unwield {
    pub(crate) item: Entity,
}

impl ActorChange for Unwield {}

#[derive(Clone, Debug)]
pub(crate) struct Pickup {
    pub(crate) item: Entity,
}

impl ActorChange for Pickup {}

/** Redundantly named to avoid confusion */
#[derive(Clone, Debug)]
pub(crate) struct MoveItem {
    pub(crate) item: Entity,
    pub(crate) to: Nbor,
}

impl ActorChange for MoveItem {}

/** Redundantly named to avoid confusion */
#[derive(Clone, Debug)]
pub(crate) struct ExamineItem {
    pub(crate) item: Entity,
}

impl ActorChange for ExamineItem {}

#[derive(Clone, Debug)]
pub(crate) struct ChangePace;

impl ActorChange for ChangePace {}
