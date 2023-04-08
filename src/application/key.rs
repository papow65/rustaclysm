use bevy::{
    input::keyboard::{KeyCode, KeyboardInput},
    prelude::Input,
};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ctrl {
    With,
    Without,
}

impl From<bool> for Ctrl {
    fn from(ctrl: bool) -> Self {
        if ctrl {
            Self::With
        } else {
            Self::Without
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Shift {
    With,
    Without,
}

impl From<bool> for Shift {
    fn from(shift: bool) -> Self {
        if shift {
            Self::With
        } else {
            Self::Without
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    KeyCode(KeyCode), // preferred, but not always available
    ScanCode(u32),
}

impl Key {
    pub(crate) const PIPE: Self = Self::ScanCode(2);
    pub(crate) const LESS_THAN: Self = Self::ScanCode(86);
}

impl fmt::Display for Key {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::KeyCode(key_code) => write!(formatter, "{key_code:?}"),
            Self::ScanCode(code) => write!(formatter, "(scancode {code:?})"),
        }
    }
}

impl From<&KeyboardInput> for Key {
    fn from(input: &KeyboardInput) -> Self {
        if let Some(key_code) = input.key_code {
            Self::KeyCode(key_code)
        } else {
            Self::ScanCode(input.scan_code)
        }
    }
}

pub(crate) struct KeyCombo(pub(crate) Ctrl, pub(crate) Shift, pub(crate) Key);

impl KeyCombo {
    pub(crate) fn new(input: &KeyboardInput, keys: &Input<KeyCode>) -> Self {
        Self(
            Ctrl::from(keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl)),
            Shift::from(keys.pressed(KeyCode::LShift) || keys.pressed(KeyCode::RShift)),
            Key::from(input),
        )
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "{}{}{}",
            if self.0 == Ctrl::With { "ctrl+" } else { "" },
            if self.1 == Shift::With { "shift+" } else { "" },
            self.2
        )
    }
}
