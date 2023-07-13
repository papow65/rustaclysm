use crate::prelude::{Ctrl, KeyCombo, Nbor};
use bevy::prelude::{Entity, KeyCode};

use super::HorizontalDirection;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CancelContext {
    State,
    Action,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    pub(crate) fn to_nbor(self) -> Nbor {
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

impl TryFrom<&KeyCombo> for PlayerDirection {
    type Error = ();

    fn try_from(combo: &KeyCombo) -> Result<Self, ()> {
        Ok(match combo {
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad1) => Self::CloserLeft,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad2) => Self::Closer,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad3) => Self::CloserRight,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad4) => Self::Left,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad5) => Self::Here,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad6) => Self::Right,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad7) => Self::AwayLeft,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad8) => Self::Away,
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Numpad9) => Self::AwayRight,
            KeyCombo::Character('<') => PlayerDirection::Above,
            KeyCombo::Character('>') => PlayerDirection::Below,
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum QueuedInstruction {
    Offset(PlayerDirection),
    Wield(Entity),
    Unwield(Entity),
    Pickup(Entity),
    Dump(Entity),
    Attack,
    Smash,
    Close,
    Wait,
    Sleep,
    SwitchRunning,
    ExamineItem(Entity),
    ExaminePos,
    ExamineZoneLevel,
    CancelAction,
    /** Set automatically */
    Interrupted,
    /** Set automatically */
    Finished,
}

impl TryFrom<&KeyCombo> for QueuedInstruction {
    type Error = ();

    fn try_from(combo: &KeyCombo) -> Result<Self, ()> {
        match combo {
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Escape) => Ok(Self::CancelAction),
            KeyCombo::Character('|') => Ok(Self::Wait),
            KeyCombo::Character('$') => Ok(Self::Sleep),
            KeyCombo::Character('a') => Ok(Self::Attack),
            KeyCombo::Character('s') => Ok(Self::Smash),
            KeyCombo::Character('c') => Ok(Self::Close),
            KeyCombo::Character('x') => Ok(Self::ExaminePos),
            KeyCombo::Character('X') => Ok(Self::ExamineZoneLevel),
            KeyCombo::Character('+') => Ok(Self::SwitchRunning),
            _ => PlayerDirection::try_from(combo).map(Self::Offset),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ZoomDirection {
    In,
    Out,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    Queued(QueuedInstruction),
    Quit,
    MainMenu,
    CancelState,
    Inventory,
    ToggleElevation,
    ToggleHelp,
    Zoom(ZoomDirection),
}

impl TryFrom<(&KeyCombo, CancelContext)> for Instruction {
    type Error = ();

    fn try_from((combo, context): (&KeyCombo, CancelContext)) -> Result<Self, ()> {
        match (combo, context) {
            (KeyCombo::KeyCode(Ctrl::With, KeyCode::C | KeyCode::Q), _) => Ok(Self::Quit),
            (KeyCombo::KeyCode(Ctrl::Without, KeyCode::Escape), CancelContext::State) => {
                Ok(Self::CancelState)
            }
            (KeyCombo::KeyCode(Ctrl::Without, KeyCode::F1), _) => Ok(Self::ToggleHelp),
            (KeyCombo::KeyCode(Ctrl::Without, KeyCode::F12), _) => Ok(Self::MainMenu),
            (KeyCombo::Character('i'), _) => Ok(Self::Inventory),
            (KeyCombo::Character('Z'), _) => Ok(Self::Zoom(ZoomDirection::Out)),
            (KeyCombo::Character('z'), _) => Ok(Self::Zoom(ZoomDirection::In)),
            (KeyCombo::Character('h'), _) => Ok(Self::ToggleElevation),
            _ => QueuedInstruction::try_from(combo).map(Self::Queued),
        }
    }
}
