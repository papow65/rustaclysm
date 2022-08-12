use crate::prelude::{Ctrl, Key, KeyCombo, Level, Pos, Shift};
use bevy::prelude::KeyCode;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
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
    pub const fn get_relative_pos(&self) -> Pos {
        Pos::new(
            match self {
                Self::CloserLeft | Self::Closer | Self::CloserRight => -1,
                Self::AwayLeft | Self::Away | Self::AwayRight => 1,
                _ => 0,
            },
            Level::new(match self {
                Self::Above => 1,
                Self::Below => -1,
                _ => 0,
            }),
            match self {
                Self::CloserLeft | Self::Left | Self::AwayLeft => -1,
                Self::CloserRight | Self::Right | Self::AwayRight => 1,
                _ => 0,
            },
        )
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum QueuedInstruction {
    Offset(Direction),
    Pickup,
    Dump,
    Attack,
    Smash,
    SwitchRunning,
    ExaminePos,
    ExamineZoneLevel,
    Cancel,
}

impl TryFrom<&KeyCombo> for QueuedInstruction {
    type Error = ();

    fn try_from(combo: &KeyCombo) -> Result<Self, ()> {
        match combo {
            KeyCombo(Ctrl::Without, shift, Key::LESS_THAN) => {
                Ok(Self::Offset(if shift == &Shift::With {
                    Direction::Above
                } else {
                    Direction::Below
                }))
            }
            KeyCombo(Ctrl::Without, Shift::Without, Key::KeyCode(key_code)) => Ok(match key_code {
                KeyCode::B => Self::Pickup,
                KeyCode::V => Self::Dump,
                KeyCode::A => Self::Attack,
                KeyCode::S => Self::Smash,
                KeyCode::X => Self::ExaminePos,
                KeyCode::M => Self::ExamineZoneLevel,
                KeyCode::NumpadAdd => Self::SwitchRunning,
                KeyCode::Escape => Self::Cancel,
                key_code => {
                    return Direction::try_from(key_code).map(Self::Offset);
                }
            }),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZoomDirection {
    In,
    Out,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Queued(QueuedInstruction),
    Quit,
    ToggleHelp,
    Zoom(ZoomDirection),
}

impl TryFrom<&KeyCombo> for Instruction {
    type Error = ();

    fn try_from(combo: &KeyCombo) -> Result<Self, ()> {
        match combo {
            KeyCombo(Ctrl::With, _, Key::KeyCode(KeyCode::C | KeyCode::D | KeyCode::Q)) => {
                Ok(Self::Quit)
            }
            KeyCombo(Ctrl::Without, shift, Key::KeyCode(KeyCode::Z)) => {
                Ok(Self::Zoom(if shift == &Shift::With {
                    ZoomDirection::Out
                } else {
                    ZoomDirection::In
                }))
            }
            KeyCombo(Ctrl::Without, Shift::Without, Key::KeyCode(KeyCode::H)) => {
                Ok(Self::ToggleHelp)
            }
            _ => QueuedInstruction::try_from(combo).map(Self::Queued),
        }
    }
}
