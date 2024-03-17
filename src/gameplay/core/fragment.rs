use crate::prelude::Pos;
use bevy::prelude::{Color, TextSection, TextStyle};
use regex::Regex;
use std::{cmp::Eq, fmt, sync::OnceLock};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Positioning {
    Pos(Pos),
    Player,
    #[default]
    None,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Fragment {
    pub(crate) text: String,
    pub(crate) color: Option<Color>,
    pub(crate) positioning: Positioning,
}

impl Fragment {
    pub(crate) fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color: None,
            positioning: Positioning::None,
        }
    }

    pub(crate) fn colorized<S>(text: S, color: Color) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color: Some(color),
            positioning: Positioning::None,
        }
    }

    pub(crate) fn positioned<S>(text: S, color: Color, pos: Pos) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color: Some(color),
            positioning: Positioning::Pos(pos),
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
    pub(crate) fn as_text_sections(&self, text_style: &TextStyle) -> Vec<TextSection> {
        self.fragments.iter().filter(|f| !f.text.is_empty()).fold(
            Vec::new(),
            |mut text_sections, f| {
                let text_style = text_style.clone();

                text_sections.push(TextSection {
                    value: if text_sections
                        .last()
                        .map_or(false, |l| Self::space_between(&l.value, &f.text))
                    {
                        format!(" {}", f.text)
                    } else {
                        f.text.clone()
                    },
                    style: TextStyle {
                        font: text_style.font,
                        font_size: text_style.font_size,
                        color: f.color.unwrap_or(text_style.color),
                    },
                });
                text_sections
            },
        )
    }

    #[must_use]
    pub(crate) fn as_string(&self) -> String {
        self.as_text_sections(&TextStyle::default())
            .into_iter()
            .map(|text_section| text_section.value)
            .collect::<String>()
    }

    fn space_between(previous: &str, next: &str) -> bool {
        static SPACE_AFTER: OnceLock<Regex> = OnceLock::new();
        static SPACE_BEFORE: OnceLock<Regex> = OnceLock::new();

        SPACE_AFTER
            .get_or_init(|| Regex::new(r"[^(\[{ \n]$").expect("Valid regex after"))
            .is_match(previous)
            && SPACE_BEFORE
                .get_or_init(|| Regex::new(r"^[^)\]},;%\. \n]").expect("Valid regex before"))
                .is_match(next)
    }
}

impl fmt::Display for Phrase {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.as_string().fmt(formatter)
    }
}

#[cfg(test)]
mod container_tests {
    use super::*;

    #[test]
    fn words() {
        let phrase = Phrase::new("one")
            .add("two")
            .push(Fragment::new("three"))
            .extend(vec![Fragment::new("four"), Fragment::new("five")]);
        assert_eq!(&phrase.as_string(), "one two three four five");
    }

    #[test]
    fn punctuation() {
        let phrase = Phrase::new("A")
            .add(",")
            .push(Fragment::new("B"))
            .add(".")
            .extend(vec![
                Fragment::new("C"),
                Fragment::new(";"),
                Fragment::new("D"),
            ]);
        assert_eq!(&phrase.as_string(), "A, B. C; D");
    }

    #[test]
    fn brackets() {
        let phrase = Phrase::new("(a)")
            .add("(")
            .add("b")
            .add(")")
            .push(Fragment::new("[c]"))
            .push(Fragment::new("["))
            .push(Fragment::new("d"))
            .push(Fragment::new("]"))
            .extend(vec![
                Fragment::new("{e}"),
                Fragment::new("{"),
                Fragment::new("f"),
                Fragment::new("}"),
                Fragment::new("(g)"),
            ]);
        assert_eq!(&phrase.as_string(), "(a) (b) [c] [d] {e} {f} (g)");
    }

    #[test]
    fn empty() {
        let phrase = Phrase::new("one")
            .add("")
            .add("{")
            .add("")
            .add("two")
            .add("")
            .add("}")
            .add("")
            .add("three");
        assert_eq!(&phrase.as_string(), "one {two} three");
    }

    #[test]
    fn mix() {
        let phrase = Phrase::new("one")
            .add("2")
            .add(",")
            .push(Fragment::new("three"))
            .extend(vec![Fragment::new("(four)"), Fragment::new("five")])
            .add("6")
            .add("%");
        assert_eq!(&phrase.as_string(), "one 2, three (four) five 6%");
    }
}
