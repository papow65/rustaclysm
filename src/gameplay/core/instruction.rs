use crate::gameplay::{Fragment, HorizontalDirection, Nbor, RecipeSituation};
use crate::keyboard::Key;
use bevy::{input::keyboard::KeyCode, prelude::Entity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CancelHandling {
    Queued,
    Menu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl TryFrom<Key> for PlayerDirection {
    type Error = ();

    fn try_from(key: Key) -> Result<Self, ()> {
        Ok(match key {
            Key::Code(KeyCode::Numpad1) => Self::CloserLeft,
            Key::Code(KeyCode::Numpad2) => Self::Closer,
            Key::Code(KeyCode::Numpad3) => Self::CloserRight,
            Key::Code(KeyCode::Numpad4) => Self::Left,
            Key::Code(KeyCode::Numpad5) => Self::Here,
            Key::Code(KeyCode::Numpad6) => Self::Right,
            Key::Code(KeyCode::Numpad7) => Self::AwayLeft,
            Key::Code(KeyCode::Numpad8) => Self::Away,
            Key::Code(KeyCode::Numpad9) => Self::AwayRight,
            Key::Character('<') => Self::Above,
            Key::Character('>') => Self::Below,
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Interruption {
    Danger(Fragment),
    LowStamina,
    Finished,
}

/// All instructions related to player character actions
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum QueuedInstruction {
    Offset(PlayerDirection),
    Wield(Entity),
    Unwield(Entity),
    Pickup(Entity),
    Dump(Entity, HorizontalDirection),
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
    ChangePace,
    ExamineItem(Entity),
    CancelAction,
    /// Set automatically
    Interrupt(Interruption),
}

impl QueuedInstruction {
    pub(crate) const fn held_key_allowed(&self) -> bool {
        matches!(self, Self::Offset(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ZoomDistance {
    Close,
    Far,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ZoomDirection {
    In,
    Out,
}
