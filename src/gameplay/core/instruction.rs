use crate::prelude::{Ctrl, KeyCombo, Nbor};
use bevy::prelude::{Entity, KeyCode};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Here,
    Away,
    AwayRight,
    Right,
    CloserRight,
    Closer,
    CloserLeft,
    Left,
    AwayLeft,
    Above,
    Below,
}

impl Direction {
    pub(crate) fn to_nbor(self) -> Nbor {
        match self {
            Self::Above => Nbor::Up,
            Self::Below => Nbor::Down,
            Self::Here => Nbor::Here,
            _ => Nbor::try_horizontal(
                match self {
                    Self::CloserLeft | Self::Left | Self::AwayLeft => -1,
                    Self::CloserRight | Self::Right | Self::AwayRight => 1,
                    _ => 0,
                },
                match self {
                    Self::AwayLeft | Self::Away | Self::AwayRight => -1,
                    Self::CloserLeft | Self::Closer | Self::CloserRight => 1,
                    _ => 0,
                },
            )
            .unwrap_or_else(|| panic!("{self:?} should have a matching nbor")),
        }
    }
}

impl TryFrom<&KeyCombo> for Direction {
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
            KeyCombo::Character('<') => Direction::Above,
            KeyCombo::Character('>') => Direction::Below,
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum QueuedInstruction {
    Offset(Direction),
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
    Inventory,
    Cancel,
    /** Set automatically */
    Interrupted,
    /** Set automatically */
    Finished,
}

impl TryFrom<&KeyCombo> for QueuedInstruction {
    type Error = ();

    fn try_from(combo: &KeyCombo) -> Result<Self, ()> {
        match combo {
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::Escape) => Ok(Self::Cancel),
            KeyCombo::Character('i') => Ok(Self::Inventory),
            KeyCombo::Character('|') => Ok(Self::Wait),
            KeyCombo::Character('$') => Ok(Self::Sleep),
            KeyCombo::Character('a') => Ok(Self::Attack),
            KeyCombo::Character('s') => Ok(Self::Smash),
            KeyCombo::Character('c') => Ok(Self::Close),
            KeyCombo::Character('x') => Ok(Self::ExaminePos),
            KeyCombo::Character('X') => Ok(Self::ExamineZoneLevel),
            KeyCombo::Character('+') => Ok(Self::SwitchRunning),
            _ => Direction::try_from(combo).map(Self::Offset),
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
    ToggleElevation,
    ToggleHelp,
    Zoom(ZoomDirection),
}

impl TryFrom<&KeyCombo> for Instruction {
    type Error = ();

    fn try_from(combo: &KeyCombo) -> Result<Self, ()> {
        match combo {
            KeyCombo::KeyCode(Ctrl::With, KeyCode::C | KeyCode::Q) => Ok(Self::Quit),
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::F1) => Ok(Self::ToggleHelp),
            KeyCombo::KeyCode(Ctrl::Without, KeyCode::F12) => Ok(Self::MainMenu),
            KeyCombo::Character('Z') => Ok(Self::Zoom(ZoomDirection::Out)),
            KeyCombo::Character('z') => Ok(Self::Zoom(ZoomDirection::In)),
            KeyCombo::Character('h') => Ok(Self::ToggleElevation),
            _ => QueuedInstruction::try_from(combo).map(Self::Queued),
        }
    }
}
