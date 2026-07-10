use bevy::prelude::TextColor;
use hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, WARN_TEXT_COLOR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Severity {
    /// For neutral informaion
    Neutral,

    /// For danger to the player character
    Danger,

    /// For actions that can't be performed as instructed
    ImpossibleAction,

    /// For errors caused by the game
    Error,

    /// For positive outcomes for the player character
    Success,
}

impl Severity {
    #[must_use]
    pub const fn color_override(&self) -> Option<TextColor> {
        match self {
            Self::Neutral => None,
            Self::Danger | Self::ImpossibleAction => Some(WARN_TEXT_COLOR),
            Self::Error => Some(BAD_TEXT_COLOR),
            Self::Success => Some(GOOD_TEXT_COLOR),
        }
    }
}
