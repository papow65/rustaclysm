use crate::prelude::*;
use bevy::prelude::{Color, Event};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Severity {
    #[allow(unused)]
    Low,
    Info,
    Warn,
    Error,
}

impl Severity {
    #[must_use]
    pub(crate) const fn color(&self) -> Color {
        match self {
            Self::Low => SOFT_TEXT_COLOR,
            Self::Info => DEFAULT_TEXT_COLOR,
            Self::Warn => WARN_TEXT_COLOR,
            Self::Error => BAD_TEXT_COLOR,
        }
    }
}

/** Message shown to the player */
#[derive(Clone, Debug, PartialEq, Eq, Event)]
pub(crate) struct Message {
    pub(crate) phrase: Phrase,
    pub(crate) severity: Severity,
}

impl Message {
    #[must_use]
    pub(crate) const fn info(phrase: Phrase) -> Self {
        Self {
            phrase,
            severity: Severity::Info,
        }
    }

    #[must_use]
    pub(crate) const fn warn(phrase: Phrase) -> Self {
        Self {
            phrase,
            severity: Severity::Warn,
        }
    }

    #[must_use]
    pub(crate) const fn error(phrase: Phrase) -> Self {
        Self {
            phrase,
            severity: Severity::Error,
        }
    }
}
