use bevy::{input::keyboard::KeyCode, prelude::Input};
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

pub(crate) struct KeyCombo(pub(crate) Ctrl, pub(crate) KeyCode);

impl KeyCombo {
    pub(crate) fn new(key_code: KeyCode, keys: &Input<KeyCode>) -> Self {
        Self(
            Ctrl::from(keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl)),
            key_code,
        )
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            formatter,
            "{}{:?}",
            if self.0 == Ctrl::With { "ctrl+" } else { "" },
            self.1
        )
    }
}
