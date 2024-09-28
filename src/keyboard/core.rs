use bevy::prelude::KeyCode;

// Ctrl

pub(crate) trait CtrlState: Send + Sync + 'static {}

impl CtrlState for () {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Ctrl;

impl CtrlState for Ctrl {}

// Held

pub(crate) trait HeldState: Send + Sync + 'static {}

impl HeldState for () {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Held;

impl HeldState for Held {}

// other types

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Key {
    Character(char),
    Code(KeyCode),
}

impl From<char> for Key {
    fn from(source: char) -> Self {
        Self::Character(source)
    }
}

impl From<KeyCode> for Key {
    fn from(source: KeyCode) -> Self {
        Self::Code(source)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputChange {
    JustPressed,
    Held,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyChange {
    pub(crate) key: Key,
    pub(crate) change: InputChange,
}
