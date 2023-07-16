use crate::prelude::Pos;
use bevy::prelude::Entity;

use super::HorizontalDirection;

#[derive(Copy, Clone, Debug)]
pub(crate) enum StayDuration {
    Short,
    Long,
}

#[derive(Debug)]
pub(crate) enum Action {
    Stay {
        duration: StayDuration,
    },
    Step {
        target: Pos, // nbor pos
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
    SwitchRunning,
}
