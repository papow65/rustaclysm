use crate::prelude::{
    Amount, CddaItemName, Filthy, Fragment, ItemName, AIR_COLOR, BAD_TEXT_COLOR,
    DEFAULT_TEXT_COLOR, FILTHY_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use bevy::prelude::{Color, Component, TextSection, TextStyle};
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
    pub(crate) fn as_item(&self, amount: &Amount, filthy: Option<&Filthy>) -> Vec<Fragment> {
        let mut result = Vec::new();
        if 1 < amount.0 {
            result.push(Fragment::new(format!("{}", amount.0), DEFAULT_TEXT_COLOR));
        }
        if filthy.is_some() {
            result.push(Fragment::new("filthy", FILTHY_COLOR));
        }
        result.push(Fragment::new(
            match amount.0 {
                1 => self.name.single.clone(),
                _ => self.name.plural.clone(),
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
    pub(crate) fn air() -> Self {
        Self::from_str("the air", AIR_COLOR)
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
        self.fragments
            .into_iter()
            .enumerate()
            .map(|(index, f)| TextSection {
                value: match (index, f.text.as_bytes().first()) {
                    (0, _) | (_, Some(b'\n')) => f.text,
                    _ => String::from(" ") + &f.text,
                },
                style: TextStyle {
                    color: f.color,
                    ..fallback_style.clone()
                },
            })
            .collect()
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
