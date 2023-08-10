use crate::prelude::*;
use bevy::prelude::{Color, Component};

#[derive(Clone, Component, Debug)]
pub(crate) struct ObjectName {
    name: ItemName,
    color: Color,
}

impl ObjectName {
    #[must_use]
    pub(crate) fn single(&self) -> Fragment {
        Fragment::colorized(self.name.single.clone(), self.color)
    }

    #[must_use]
    pub(crate) fn as_item(
        &self,
        amount: Option<&Amount>,
        filthy: Option<&Filthy>,
    ) -> Vec<Fragment> {
        let amount = match amount {
            Some(Amount(n)) => *n,
            _ => 1,
        };
        let mut result = Vec::new();
        if 1 < amount {
            result.push(Fragment::new(format!("{amount}")));
        }
        if filthy.is_some() {
            result.push(Fragment::colorized("filthy", FILTHY_COLOR));
        }
        result.push(Fragment::colorized(
            if amount == 1 {
                self.name.single.clone()
            } else {
                self.name.plural.clone()
            },
            self.color,
        ));
        result
    }

    #[must_use]
    pub(crate) const fn new(name: ItemName, color: Color) -> Self {
        Self { name, color }
    }

    #[must_use]
    pub(crate) fn from_str(text: &str, color: Color) -> Self {
        Self {
            name: ItemName::from(CddaItemName::Simple(String::from(text))),
            color,
        }
    }

    #[must_use]
    pub(crate) fn corpse() -> Self {
        Self::from_str("corpse", BAD_TEXT_COLOR)
    }

    #[must_use]
    pub(crate) fn missing() -> Self {
        Self::from_str("(missing name)", BAD_TEXT_COLOR)
    }
}

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
#[derive(Component, Debug, PartialEq, Eq)]
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
