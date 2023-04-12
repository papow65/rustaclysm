use crate::prelude::{Ctrl, KeyCombo, Nbor};
use bevy::prelude::KeyCode;

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

impl TryFrom<&KeyCode> for Direction {
    type Error = ();

    fn try_from(key_code: &KeyCode) -> Result<Self, ()> {
        Ok(match key_code {
            KeyCode::Numpad1 => Self::CloserLeft,
            KeyCode::Numpad2 => Self::Closer,
            KeyCode::Numpad3 => Self::CloserRight,
            KeyCode::Numpad4 => Self::Left,
            KeyCode::Numpad5 => Self::Here,
            KeyCode::Numpad6 => Self::Right,
            KeyCode::Numpad7 => Self::AwayLeft,
            KeyCode::Numpad8 => Self::Away,
            KeyCode::Numpad9 => Self::AwayRight,
            KeyCode::R => Self::Above,
            KeyCode::F => Self::Below,
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum QueuedInstruction {
    Offset(Direction),
    Wield,
    Pickup,
    Dump,
    Attack,
    Smash,
    Close,
    Wait,
    Sleep,
    SwitchRunning,
    ExaminePos,
    ExamineZoneLevel,
    Cancel,
    /** Set automatically */
    Interrupted,
    /** Set automatically */
    Finished,
}

impl TryFrom<&KeyCode> for QueuedInstruction {
    type Error = ();

    fn try_from(key_code: &KeyCode) -> Result<Self, ()> {
        match key_code {
            KeyCode::Escape => Ok(Self::Cancel),
            _ => Direction::try_from(key_code).map(Self::Offset),
        }
    }
}

impl TryFrom<char> for QueuedInstruction {
    type Error = ();

    fn try_from(input: char) -> Result<Self, ()> {
        match input {
            '<' => Ok(Direction::Above).map(Self::Offset),
            '>' => Ok(Direction::Below).map(Self::Offset),
            '|' => Ok(Self::Wait),
            '$' => Ok(Self::Sleep),
            'w' => Ok(Self::Wield),
            'b' => Ok(Self::Pickup),
            'v' => Ok(Self::Dump),
            'a' => Ok(Self::Attack),
            's' => Ok(Self::Smash),
            'c' => Ok(Self::Close),
            'x' => Ok(Self::ExaminePos),
            'm' => Ok(Self::ExamineZoneLevel),
            '+' => Ok(Self::SwitchRunning),
            _ => Err(()),
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
            KeyCombo(Ctrl::With, KeyCode::C | KeyCode::Q) => Ok(Self::Quit),
            KeyCombo(_, KeyCode::F1) => Ok(Self::ToggleHelp),
            KeyCombo(_, KeyCode::F12) => Ok(Self::MainMenu),
            _ => QueuedInstruction::try_from(&combo.1).map(Self::Queued),
        }
    }
}

impl TryFrom<char> for Instruction {
    type Error = ();

    fn try_from(input: char) -> Result<Self, ()> {
        match input {
            'Z' => Ok(Self::Zoom(ZoomDirection::Out)),
            'z' => Ok(Self::Zoom(ZoomDirection::In)),
            'h' => Ok(Self::ToggleElevation),
            _ => QueuedInstruction::try_from(input).map(Self::Queued),
        }
    }
}
