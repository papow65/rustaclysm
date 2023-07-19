use crate::prelude::ObjectName;
use bevy::prelude::{Color, TextSection, TextStyle};
use regex::Regex;
use std::{cmp::Eq, fmt};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Fragment {
    pub(crate) text: String,
    pub(crate) color: Option<Color>,
}

impl Fragment {
    pub(crate) fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color: None,
        }
    }

    pub(crate) fn colorized<S>(text: S, color: Color) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color: Some(color),
        }
    }
}

// The floats in color are unimportant and often come from constants
impl Eq for Fragment {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Phrase {
    pub(crate) fragments: Vec<Fragment>,
}

impl Phrase {
    #[must_use]
    pub(crate) fn new(text: impl Into<String>) -> Self {
        Self {
            fragments: vec![Fragment::new(text.into())],
        }
    }

    #[must_use]
    pub(crate) fn from_name(name: &ObjectName) -> Self {
        Self {
            fragments: vec![name.single()],
        }
    }

    #[must_use]
    pub(crate) fn from_fragment(fragment: Fragment) -> Self {
        Self {
            fragments: vec![fragment],
        }
    }

    #[must_use]
    pub(crate) fn from_fragments(fragments: Vec<Fragment>) -> Self {
        Self { fragments }
    }

    #[must_use]
    pub(crate) fn add(self, text: impl Into<String>) -> Self {
        self.push(Fragment::new(text.into()))
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
    pub(crate) fn into_text_sections(self, text_style: &TextStyle) -> Vec<TextSection> {
        let no_space_after = Regex::new(r"[( \n]$").expect("Valid regex after");
        let no_space_before = Regex::new(r"^[) \n]").expect("Valid regex before");

        self.fragments
            .into_iter()
            .filter(|f| !f.text.is_empty())
            .fold(Vec::new(), |mut text_sections, f| {
                let text_style = text_style.clone();

                text_sections.push(TextSection {
                    value: if text_sections
                        .last()
                        .map_or(true, |l| no_space_after.is_match(&l.value))
                        || no_space_before.is_match(&f.text)
                    {
                        f.text
                    } else {
                        format!(" {}", f.text)
                    },
                    style: TextStyle {
                        font: text_style.font,
                        font_size: text_style.font_size,
                        color: f.color.unwrap_or(text_style.color),
                    },
                });
                text_sections
            })
    }
}

impl fmt::Display for Phrase {
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
