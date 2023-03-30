use crate::prelude::Pos;

#[derive(Debug)]
pub(crate) enum Action {
    Stay,
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
