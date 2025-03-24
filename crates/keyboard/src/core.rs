use bevy::prelude::KeyCode;

// Ctrl

pub trait CtrlState: Clone + Default + Send + Sync + 'static {}

impl CtrlState for () {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Ctrl;

impl CtrlState for Ctrl {}

// Held

pub trait HeldState: Clone + Default + Send + Sync + 'static {}

impl HeldState for () {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Held;

impl HeldState for Held {}

// other types

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
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
pub enum InputChange {
    JustPressed,
    Held,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyChange {
    pub key: Key,
    pub change: InputChange,
}
