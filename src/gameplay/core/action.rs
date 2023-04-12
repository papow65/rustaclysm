use crate::prelude::Pos;

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
    Wield, // start wielding
    Pickup,
    Dump,
    SwitchRunning,
}
