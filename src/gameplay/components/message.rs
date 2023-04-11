use crate::prelude::{
    Amount, CddaItemName, Filthy, Fragment, ItemName, BAD_TEXT_COLOR, DEFAULT_TEXT_COLOR,
    FILTHY_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use bevy::prelude::{Color, Component, TextSection, TextStyle};
use regex::Regex;
use std::fmt;

#[derive(Component, Debug)]
pub(crate) struct ObjectName {
    name: ItemName,
    color: Color,
}

impl ObjectName {
    #[must_use]
    pub(crate) fn single(&self) -> Fragment {
        Fragment {
            text: self.name.single.clone(),
            color: self.color,
        }
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
            result.push(Fragment::new(format!("{amount}"), DEFAULT_TEXT_COLOR));
        }
        if filthy.is_some() {
            result.push(Fragment::new("filthy", FILTHY_COLOR));
        }
        result.push(Fragment::new(
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
    pub(crate) fn new(name: ItemName, color: Color) -> Self {
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
    Low,
    Info,
    Warn,
    Error,
}

impl Severity {
    #[must_use]
    pub(crate) fn color(&self) -> Color {
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
    pub(crate) fragments: Vec<Fragment>,
    pub(crate) severity: Severity,
}

impl Message {
    #[must_use]
    fn new(severity: Severity) -> Self {
        Self {
            fragments: Vec::new(),
            severity,
        }
    }

    #[must_use]
    pub(crate) fn info() -> Self {
        Self::new(Severity::Info)
    }

    #[must_use]
    pub(crate) fn warn() -> Self {
        Self::new(Severity::Warn)
    }

    #[must_use]
    pub(crate) fn error() -> Self {
        Self::new(Severity::Error)
    }

    #[must_use]
    pub(crate) fn str(self, s: &str) -> Self {
        self.add(String::from(s))
    }

    #[must_use]
    pub(crate) fn add(self, text: String) -> Self {
        let color = self.severity.color();
        self.push(Fragment::new(text, color))
    }

    #[must_use]
    pub(crate) fn push(mut self, fragment: Fragment) -> Self {
        self.fragments.push(fragment);
        self
    }

    #[must_use]
    pub(crate) fn extend(mut self, fragments: Vec<Fragment>) -> Self {
        self.fragments.extend(fragments);
        self
    }

    #[must_use]
    pub(crate) fn into_text_sections(self, fallback_style: &TextStyle) -> Vec<TextSection> {
        let no_space_after = Regex::new(r"[( \n]$").expect("Valid regex after");
        let no_space_before = Regex::new(r"^[) \n]").expect("Valid regex before");

        self.fragments
            .into_iter()
            .filter(|f| !f.text.is_empty())
            .fold(Vec::new(), |mut vec, f| {
                vec.push(TextSection {
                    value: if vec
                        .last()
                        .map_or(true, |l| no_space_after.is_match(&l.value))
                        || no_space_before.is_match(&f.text)
                    {
                        f.text
                    } else {
                        format!(" {}", f.text)
                    },
                    style: TextStyle {
                        color: f.color,
                        ..fallback_style.clone()
                    },
                });
                vec
            })
    }
}

impl fmt::Display for Message {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.fragments
            .iter()
            .map(|fragment| &fragment.text)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
            .fmt(formatter)
    }
}
