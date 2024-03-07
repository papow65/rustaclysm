use crate::prelude::{InputChange, Key, KeyCombo, Nbor};
use bevy::{input::keyboard::KeyCode, prelude::Entity};

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

/** All instructions where the order of execution matters */
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum QueuedInstruction {
    Offset(PlayerDirection),
    Wield(Entity),
    Unwield(Entity),
    Pickup(Entity),
    Dump(Entity, HorizontalDirection),
    Attack,
    Smash,
    Pulp,
    Close,
    Drag,
    Wait,
    Sleep,
    ToggleAutoTravel,
    ToggleAutoDefend,
    ChangePace,
    ExamineItem(Entity),
    ExaminePos,
    ExamineZoneLevel,
    CancelAction,
    /** Set automatically */
    Interrupted,
    /** Set automatically */
    Finished,
}

impl TryFrom<Key> for QueuedInstruction {
    type Error = ();

    fn try_from(key: Key) -> Result<Self, ()> {
        match key {
            Key::Code(KeyCode::Escape) => Ok(Self::CancelAction),
            Key::Character('|') => Ok(Self::Wait),
            Key::Character('$') => Ok(Self::Sleep),
            Key::Character('a') => Ok(Self::Attack),
            Key::Character('s') => Ok(Self::Smash),
            Key::Character('p') => Ok(Self::Pulp),
            Key::Character('c') => Ok(Self::Close),
            Key::Character('\\') => Ok(Self::Drag),
            Key::Character('G') => Ok(Self::ToggleAutoTravel),
            Key::Code(KeyCode::Tab) => Ok(Self::ToggleAutoDefend),
            Key::Character('x') => Ok(Self::ExaminePos),
            Key::Character('X') => Ok(Self::ExamineZoneLevel),
            Key::Character('+') => Ok(Self::ChangePace),
            _ => PlayerDirection::try_from(key).map(Self::Offset),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ZoomDistance {
    Close,
    Far,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ZoomDirection {
    In,
    Out,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    Queued(QueuedInstruction),
    ShowMainMenu,
    ShowGameplayMenu,
    Inventory,
    ToggleMap(ZoomDistance),
    ToggleElevation,
    ToggleHelp,
    Zoom(ZoomDirection),
    ResetCameraAngle,
}

impl TryFrom<(&KeyCombo, CancelContext)> for Instruction {
    type Error = ();

    fn try_from((combo, context): (&KeyCombo, CancelContext)) -> Result<Self, ()> {
        match (combo.key, combo.change, context) {
            (Key::Code(KeyCode::Escape), InputChange::Held, _) => Err(()),
            (Key::Code(KeyCode::Escape), InputChange::JustPressed, CancelContext::State) => {
                Ok(Self::ShowGameplayMenu)
            }
            (Key::Code(KeyCode::F1), InputChange::JustPressed, _) => Ok(Self::ToggleHelp),
            (Key::Code(KeyCode::F12), InputChange::JustPressed, _) => Ok(Self::ShowMainMenu),
            (Key::Character('m'), InputChange::JustPressed, _) => {
                Ok(Self::ToggleMap(ZoomDistance::Close))
            }
            (Key::Character('M'), InputChange::JustPressed, _) => {
                Ok(Self::ToggleMap(ZoomDistance::Far))
            }
            (Key::Character('i'), InputChange::JustPressed, _) => Ok(Self::Inventory),
            (Key::Character('Z'), InputChange::JustPressed, _) => {
                Ok(Self::Zoom(ZoomDirection::Out))
            }
            (Key::Character('z'), InputChange::JustPressed, _) => Ok(Self::Zoom(ZoomDirection::In)),
            (Key::Character('h'), InputChange::JustPressed, _) => Ok(Self::ToggleElevation),
            (Key::Character('0'), InputChange::JustPressed, _) => Ok(Self::ResetCameraAngle),
            _ => QueuedInstruction::try_from(combo.key).map(Self::Queued),
        }
    }
}
