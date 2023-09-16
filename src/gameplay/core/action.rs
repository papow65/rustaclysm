use crate::prelude::{ActorChange, Pos};
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
        to: Pos, // nbor pos
    },
    Attack {
        target: Pos, // nbor pos
    },
    Smash {
        target: Pos, // nbor pos
    },
    Close {
        target: Pos, // nbor pos
    },
    Wield {
        entity: Entity,
    },
    Unwield {
        entity: Entity,
    },
    Pickup {
        entity: Entity,
    },
    Dump {
        entity: Entity,
        direction: HorizontalDirection,
    },
    ExamineItem {
        entity: Entity,
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
    pub(crate) to: Pos, // nbor pos
}

impl ActorChange for Step {}

#[derive(Clone, Debug)]
pub(crate) struct Attack {
    pub(crate) target: Pos, // nbor pos
}

impl ActorChange for Attack {}

#[derive(Clone, Debug)]
pub(crate) struct Smash {
    pub(crate) target: Pos, // nbor pos
}

impl ActorChange for Smash {}

#[derive(Clone, Debug)]
pub(crate) struct Close {
    pub(crate) target: Pos, // nbor pos
}

impl ActorChange for Close {}

#[derive(Clone, Debug)]
pub(crate) struct Wield {
    pub(crate) entity: Entity,
}

impl ActorChange for Wield {}

#[derive(Clone, Debug)]
pub(crate) struct Unwield {
    pub(crate) entity: Entity,
}

impl ActorChange for Unwield {}

#[derive(Clone, Debug)]
pub(crate) struct Pickup {
    pub(crate) entity: Entity,
}

impl ActorChange for Pickup {}

#[derive(Clone, Debug)]
pub(crate) struct Dump {
    pub(crate) entity: Entity,
    pub(crate) direction: HorizontalDirection,
}

impl ActorChange for Dump {}

#[derive(Clone, Debug)]
pub(crate) struct ExamineItem {
    pub(crate) entity: Entity,
}

impl ActorChange for ExamineItem {}

#[derive(Clone, Debug)]
pub(crate) struct ChangePace;

impl ActorChange for ChangePace {}
