use crate::gameplay::{Fragment, HorizontalDirection, Nbor, RecipeSituation};
use crate::keyboard::{InputChange, Key, KeyChange};
use bevy::{input::keyboard::KeyCode, prelude::Entity};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CancelHandling {
    Queued,
    Menu,
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
            Key::Code(KeyCode::Tab) => Ok(Self::Peek),
            Key::Character('c') => Ok(Self::Close),
            Key::Character('\\') => Ok(Self::Drag),
            Key::Character('G') => Ok(Self::ToggleAutoTravel),
            Key::Character('A') => Ok(Self::ToggleAutoDefend),
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

#[derive(Debug)]
pub(crate) enum Instruction {
    Queued(QueuedInstruction),
    ShowGameplayMenu,
    ExaminePos,
    ExamineZoneLevel,
    CraftingScreen,
    Inventory,
    ToggleMap(ZoomDistance),
    ToggleElevation,
    Zoom(ZoomDirection),
    ResetCameraAngle,
}

impl TryFrom<(KeyChange, CancelHandling)> for Instruction {
    type Error = ();

    fn try_from((key_change, context): (KeyChange, CancelHandling)) -> Result<Self, ()> {
        match (key_change.key, key_change.change, context) {
            (Key::Code(KeyCode::Escape), InputChange::Held, _) => Err(()),
            (Key::Code(KeyCode::Escape), InputChange::JustPressed, CancelHandling::Menu) => {
                Ok(Self::ShowGameplayMenu)
            }
            (Key::Character('m'), InputChange::JustPressed, _) => {
                Ok(Self::ToggleMap(ZoomDistance::Close))
            }
            (Key::Character('M'), InputChange::JustPressed, _) => {
                Ok(Self::ToggleMap(ZoomDistance::Far))
            }
            (Key::Character('x'), InputChange::JustPressed, _) => Ok(Self::ExaminePos),
            (Key::Character('X'), InputChange::JustPressed, _) => Ok(Self::ExamineZoneLevel),
            (Key::Character('&'), InputChange::JustPressed, _) => Ok(Self::CraftingScreen),
            (Key::Character('i'), InputChange::JustPressed, _) => Ok(Self::Inventory),
            (Key::Character('Z'), InputChange::JustPressed, _) => {
                Ok(Self::Zoom(ZoomDirection::Out))
            }
            (Key::Character('z'), InputChange::JustPressed, _) => Ok(Self::Zoom(ZoomDirection::In)),
            (Key::Character('h'), InputChange::JustPressed, _) => Ok(Self::ToggleElevation),
            (Key::Character('0'), InputChange::JustPressed, _) => Ok(Self::ResetCameraAngle),
            _ => QueuedInstruction::try_from(key_change.key).map(Self::Queued),
        }
    }
}
