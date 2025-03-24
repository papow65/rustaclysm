use crate::gameplay::Phrase;
use bevy::prelude::{Event, TextColor};
use hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, WARN_TEXT_COLOR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Severity {
    Info,
    Warn,
    Error,
    Success,
}

impl Severity {
    #[must_use]
    pub(crate) const fn color_override(&self) -> Option<TextColor> {
        match self {
            Self::Info => None,
            Self::Warn => Some(WARN_TEXT_COLOR),
            Self::Error => Some(BAD_TEXT_COLOR),
            Self::Success => Some(GOOD_TEXT_COLOR),
        }
    }
}

/// Message shown to the player
#[derive(Clone, Debug, PartialEq, Eq, Event)]
pub(crate) struct Message {
    pub(crate) phrase: Phrase,
    pub(crate) severity: Severity,
    pub(crate) transient: bool,
}
