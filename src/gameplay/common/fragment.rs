use crate::{gameplay::Pos, hud::GOOD_TEXT_COLOR};
use bevy::prelude::{TextColor, TextFont, TextSpan};
use regex::Regex;
use std::{cmp::Eq, fmt, sync::LazyLock};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Positioning {
    Pos(Pos),
    Player,
    #[default]
    None,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Fragment {
    pub(crate) text: String,
    pub(crate) color: Option<TextColor>,
    pub(crate) positioning: Positioning,
}

impl Fragment {
    pub(crate) fn you() -> Self {
        Self {
            text: String::from("You"),
            color: Some(GOOD_TEXT_COLOR),
            positioning: Positioning::Player,
        }
    }

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

    pub(crate) fn colorized<S>(text: S, color: TextColor) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color: Some(color),
            positioning: Positioning::None,
        }
    }

    pub(crate) fn positioned<S>(text: S, color: TextColor, pos: Pos) -> Self
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

impl PartialEq for Fragment {
    fn eq(&self, other: &Self) -> bool {
        // The floats in color are unimportant and often come from constants
        self.text == other.text && self.positioning == other.positioning
    }
}

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
    pub(crate) const fn from_fragments(fragments: Vec<Fragment>) -> Self {
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
    pub(crate) fn extend(mut self, fragments: impl IntoIterator<Item = Fragment>) -> Self {
        self.fragments.extend(fragments);
        self
    }

    #[must_use]
    pub(crate) fn as_text_sections(
        &self,
        text_color: TextColor,
        text_font: &TextFont,
    ) -> Vec<(TextSpan, TextColor, TextFont)> {
        self.fragments.iter().filter(|f| !f.text.is_empty()).fold(
            Vec::new(),
            |mut text_sections, f| {
                text_sections.push((
                    TextSpan(
                        if text_sections
                            .last()
                            .map_or(false, |l| Self::space_between(&l.0 .0, &f.text))
                        {
                            format!(" {}", f.text)
                        } else {
                            f.text.clone()
                        },
                    ),
                    f.color.unwrap_or(text_color),
                    text_font.clone(),
                ));
                text_sections
            },
        )
    }

    #[must_use]
    pub(crate) fn as_string(&self) -> String {
        self.as_text_sections(TextColor::WHITE, &TextFont::default())
            .into_iter()
            .map(|text_section| text_section.0 .0)
            .collect::<String>()
    }

    fn space_between(previous: &str, next: &str) -> bool {
        static SPACE_AFTER: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"[^(\[{ \n]$").expect("Valid regex after"));

        // Don't add a space before '.' when it's used as the end of a sentence
        // Add a space before '.' when it's used as the start of a name, like '.22'.
        static SPACE_BEFORE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^([^)\]},;%\. \n]|\.[^ ])").expect("Valid regex before"));

        SPACE_AFTER.is_match(previous) && SPACE_BEFORE.is_match(next)
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
                Fragment::new(".E"),
                Fragment::new(". F"),
                Fragment::new("\nG"),
            ]);
        assert_eq!(&phrase.as_string(), "A, B. C; D .E. F\nG");
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
