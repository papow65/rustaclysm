use crate::gameplay::{DebugText, Pos};
use crate::hud::{
    FILTHY_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use bevy::prelude::{TextColor, TextSpan};
use regex::Regex;
use std::{cmp::Eq, fmt, sync::LazyLock};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Positioning {
    Pos(Pos),
    Player,
    #[default]
    None,
}

#[must_use]
#[derive(Clone, Debug, Default)]
pub(crate) struct Fragment {
    pub(crate) text: String,
    pub(crate) color: TextColor,
    pub(crate) positioning: Positioning,
    pub(crate) debug: bool,
}

impl Fragment {
    pub(crate) fn you() -> Self {
        Self {
            text: String::from("You"),
            color: GOOD_TEXT_COLOR,
            positioning: Positioning::Player,
            debug: false,
        }
    }

    pub(crate) fn soft<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, SOFT_TEXT_COLOR)
    }

    pub(crate) fn hard<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, HARD_TEXT_COLOR)
    }

    pub(crate) fn good<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, GOOD_TEXT_COLOR)
    }

    pub(crate) fn filthy<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, FILTHY_COLOR)
    }

    pub(crate) fn warn<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, WARN_TEXT_COLOR)
    }

    pub(crate) fn colorized<S>(text: S, color: TextColor) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color,
            positioning: Positioning::None,
            debug: false,
        }
    }

    pub(crate) const fn positioned(mut self, pos: Pos) -> Self {
        self.positioning = Positioning::Pos(pos);
        self
    }

    pub(crate) const fn debug(mut self) -> Self {
        self.debug = true;
        self
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
            fragments: vec![Fragment::hard(text.into())],
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
    pub(crate) fn soft(self, text: impl Into<String>) -> Self {
        self.push(Fragment::soft(text.into()))
    }

    #[must_use]
    pub(crate) fn hard(self, text: impl Into<String>) -> Self {
        self.push(Fragment::hard(text.into()))
    }

    #[must_use]
    pub(crate) fn debug(self, text: impl Into<String>) -> Self {
        self.push(Fragment::soft(text.into()).debug())
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
    pub(crate) fn color_override(mut self, color_override: Option<TextColor>) -> Self {
        if let Some(color_override) = color_override {
            for fragment in &mut self.fragments {
                if fragment.color.0 == HARD_TEXT_COLOR.0 {
                    fragment.color = color_override;
                }
            }
        }
        self
    }

    #[must_use]
    pub(crate) fn as_text_sections(&self) -> Vec<(TextSpan, TextColor, Option<DebugText>)> {
        self.fragments.iter().filter(|f| !f.text.is_empty()).fold(
            Vec::new(),
            |mut text_sections, f| {
                text_sections.push((
                    TextSpan(
                        if text_sections
                            .last()
                            .is_some_and(|l| Self::space_between(&l.0 .0, &f.text))
                        {
                            format!(" {}", f.text)
                        } else {
                            f.text.clone()
                        },
                    ),
                    f.color,
                    f.debug.then_some(DebugText),
                ));
                text_sections
            },
        )
    }

    #[must_use]
    pub(crate) fn as_string(&self) -> String {
        self.as_text_sections()
            .into_iter()
            .map(|text_section| text_section.0 .0)
            .collect::<String>()
    }

    fn space_between(previous: &str, next: &str) -> bool {
        static SPACE_AFTER: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"[^(\[{ \n]$").expect("Valid regex after"));

        // We don't add spaces before a '/' follwed by a digit, to allow '15/20' from '15', and '/20'.
        // In other cases, we do add a space before slashes, for the damage markers on items.
        //
        // Don't add a space before '.' when it's used as the end of a sentence
        // Add a space before '.' when it's used as the start of a name, like '.22'.
        static SPACE_BEFORE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^([^)\]},;/%\. \n]|\.[^ ]|/[^0-9])").expect("Valid regex before")
        });

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
            .soft("two")
            .push(Fragment::soft("three"))
            .extend(vec![Fragment::hard("four"), Fragment::soft("five")]);
        assert_eq!(&phrase.as_string(), "one two three four five");
    }

    #[test]
    fn punctuation() {
        let phrase = Phrase::new("A")
            .hard(",")
            .push(Fragment::hard("B"))
            .soft(".")
            .extend(vec![
                Fragment::hard("C"),
                Fragment::hard(";"),
                Fragment::hard("D"),
                Fragment::hard(".E"),
                Fragment::hard(". F"),
                Fragment::hard("\nG"),
            ]);
        assert_eq!(&phrase.as_string(), "A, B. C; D .E. F\nG");
    }

    #[test]
    fn brackets() {
        let phrase = Phrase::new("(a)")
            .soft("(")
            .soft("b")
            .soft(")")
            .push(Fragment::soft("[c]"))
            .push(Fragment::soft("["))
            .push(Fragment::soft("d"))
            .push(Fragment::soft("]"))
            .extend(vec![
                Fragment::soft("{e}"),
                Fragment::soft("{"),
                Fragment::soft("f"),
                Fragment::soft("}"),
                Fragment::soft("(g)"),
            ]);
        assert_eq!(&phrase.as_string(), "(a) (b) [c] [d] {e} {f} (g)");
    }

    #[test]
    fn empty() {
        let phrase = Phrase::new("one")
            .soft("")
            .soft("{")
            .soft("")
            .soft("two")
            .soft("")
            .soft("}")
            .soft("")
            .soft("three");
        assert_eq!(&phrase.as_string(), "one {two} three");
    }

    #[test]
    fn mix() {
        let phrase = Phrase::new("one")
            .soft("2")
            .soft(",")
            .push(Fragment::soft("three"))
            .extend(vec![Fragment::soft("(four)"), Fragment::soft("five")])
            .hard("6")
            .hard("%")
            .hard("/7")
            .hard("/u8")
            .hard("9");
        assert_eq!(&phrase.as_string(), "one 2, three (four) five 6%/7 /u8 9");
    }
}
