use bevy::prelude::KeyCode;
use std::{fmt, str::Chars};

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

pub enum CharsToKeyError {
    TooManyChars(String),
    Empty,
}

impl fmt::Display for CharsToKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyChars(s) => f.write_fmt(format_args!("multiple characters ({s})")),
            Self::Empty => f.write_str("zero characters"),
        }
    }
}

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

impl TryFrom<Chars<'_>> for Key {
    type Error = CharsToKeyError;

    fn try_from(mut chars: Chars) -> Result<Self, Self::Error> {
        if let Some(char) = chars.next() {
            if let Some(next) = chars.next() {
                Err(CharsToKeyError::TooManyChars(
                    String::from(char) + &String::from(next) + &chars.collect::<String>(),
                ))
            } else {
                Ok(Self::Character(char))
            }
        } else {
            Err(CharsToKeyError::Empty)
        }
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
