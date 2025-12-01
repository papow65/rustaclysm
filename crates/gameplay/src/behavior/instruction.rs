use crate::{ChangePace, ExamineItem, Fragment, MoveItem, Pickup, RecipeSituation, Unwield, Wield};
use bevy::prelude::{Resource, warn};
use gameplay_location::{HorizontalDirection, Nbor};
use strum::VariantArray;

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq, VariantArray)]
pub(crate) enum PlayerDirection {
    Above,
    AwayLeft,
    Away,
    AwayRight,
    Left,
    Here,
    Right,
    CloserLeft,
    Closer,
    CloserRight,
    Below,
}

impl PlayerDirection {
    pub(crate) const fn to_nbor(self) -> Nbor {
        match self {
            Self::Above => Nbor::Up,
            Self::AwayLeft => Nbor::Horizontal(HorizontalDirection::NorthWest),
            Self::Away => Nbor::Horizontal(HorizontalDirection::North),
            Self::AwayRight => Nbor::Horizontal(HorizontalDirection::NorthEast),
            Self::Left => Nbor::Horizontal(HorizontalDirection::West),
            Self::Here => Nbor::Horizontal(HorizontalDirection::Here),
            Self::Right => Nbor::Horizontal(HorizontalDirection::East),
            Self::CloserLeft => Nbor::Horizontal(HorizontalDirection::SouthWest),
            Self::Closer => Nbor::Horizontal(HorizontalDirection::South),
            Self::CloserRight => Nbor::Horizontal(HorizontalDirection::SouthEast),
            Self::Below => Nbor::Down,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Interruption {
    Danger(Fragment),
    LowStamina,
    Finished,
}

/// All instructions related to player character actions
#[must_use]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum QueuedInstruction {
    Offset(PlayerDirection),
    Wield(Wield),
    Unwield(Unwield),
    Pickup(Pickup),
    MoveItem(MoveItem),
    StartCraft(RecipeSituation),
    Attack,
    Smash,
    Pulp,
    Peek,
    Close,
    Drag,
    Wait,
    Sleep,
    ToggleAutoTravel,
    ToggleAutoDefend,
    ChangePace(ChangePace),
    ExamineItem(ExamineItem),
    /// Cancelled by the player
    CancelAction,
    /// Set automatically
    Interrupt(Interruption),
}

impl QueuedInstruction {
    pub(crate) const fn held_key_allowed(&self) -> bool {
        matches!(self, Self::Offset(_))
    }
}

#[derive(Debug, Default, Resource)]
pub(crate) struct PlayerInstructions {
    /// Can be empty: when waitin for new user input, when npcs act, or when using automatic behavior
    queue: Vec<QueuedInstruction>,
}

impl PlayerInstructions {
    pub(crate) fn push(&mut self, instruction: QueuedInstruction) {
        // Wait for an instruction to be processed until adding a duplicate when holding a key down.
        if !instruction.held_key_allowed() || !self.queue.contains(&instruction) {
            self.queue.insert(0, instruction);
        }
    }

    pub(crate) fn interrupt(&mut self, interruption: Interruption) {
        self.push(QueuedInstruction::Interrupt(interruption));
    }

    pub(crate) fn pop(&mut self) -> Option<QueuedInstruction> {
        self.queue.pop()
    }

    pub(super) const fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub(crate) fn log_if_long(&self) {
        if 1 < self.queue.len() {
            warn!("Unprocessed key codes: {:?}", self.queue);
        }
    }
}
