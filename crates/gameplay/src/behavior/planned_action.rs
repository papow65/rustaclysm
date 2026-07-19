use crate::{
    Attack, ChangePace, Close, ContinueCraft, ExamineItem, MoveItem, Peek, Pickup, Pulp, Smash,
    StartCraft, Step, Unwield, Wield,
};
use gameplay_location::{CardinalDirection, HorizontalDirection, Nbor};

#[derive(Debug)]
pub(crate) enum PlannedAction {
    Stay,
    Sleep,
    Step(Step),
    Attack(Attack),
    Smash(Smash),
    Pulp(Pulp),
    Peek(Peek),
    Close(Close),
    Wield(Wield),
    Unwield(Unwield),
    Pickup(Pickup),
    /// Redundantly named to avoid confusion
    MoveItem(MoveItem),
    StartCraft(StartCraft),
    ContinueCraft(ContinueCraft),
    /// Redundantly named to avoid confusion
    ExamineItem(ExamineItem),
    ChangePace(ChangePace),
}

impl PlannedAction {
    pub(crate) const fn step(to: Nbor) -> Self {
        Self::Step(Step { to })
    }

    pub(crate) const fn attack(target: Nbor) -> Self {
        Self::Attack(Attack { target })
    }

    pub(crate) const fn smash(target: Nbor) -> Self {
        Self::Smash(Smash { target })
    }

    pub(crate) const fn pulp(target: HorizontalDirection) -> Self {
        Self::Pulp(Pulp { target })
    }

    pub(crate) const fn peek(target: CardinalDirection) -> Self {
        Self::Peek(Peek { target })
    }

    pub(crate) const fn close(target: HorizontalDirection) -> Self {
        Self::Close(Close { target })
    }
}
