use crate::Fragment;
use bevy::prelude::{TextColor, TextSpan};
use hud::{DebugText, HARD_TEXT_COLOR};
use regex::Regex;
use std::{cmp::Eq, fmt, sync::LazyLock};

/// Used for a collection of [`Fragment`s](`Fragment`)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phrase {
    pub fragments: Vec<Fragment>,
}

impl Phrase {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            fragments: vec![Fragment::hard(text.into())],
        }
    }

    #[must_use]
    pub fn from_fragment(fragment: Fragment) -> Self {
        Self {
            fragments: vec![fragment],
        }
    }

    #[must_use]
    pub const fn from_fragments(fragments: Vec<Fragment>) -> Self {
        Self { fragments }
    }

    #[must_use]
    pub fn soft(self, text: impl Into<String>) -> Self {
        self.push(Fragment::soft(text.into()))
    }

    #[must_use]
    pub fn hard(self, text: impl Into<String>) -> Self {
        self.push(Fragment::hard(text.into()))
    }

    #[must_use]
    pub fn debug(self, text: impl Into<String>) -> Self {
        self.push(Fragment::soft(text.into()).debug())
    }

    #[must_use]
    pub fn push(mut self, fragment: Fragment) -> Self {
        self.fragments.push(fragment);
        self
    }

    #[must_use]
    pub fn extend(mut self, fragments: impl IntoIterator<Item = Fragment>) -> Self {
        self.fragments.extend(fragments);
        self
    }

    #[must_use]
    pub fn color_override(mut self, color_override: Option<TextColor>) -> Self {
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
    pub fn as_text_sections(&self) -> Vec<(TextSpan, TextColor, Option<DebugText>)> {
        self.fragments.iter().filter(|f| !f.text.is_empty()).fold(
            Vec::new(),
            |mut text_sections, f| {
                text_sections.push((
                    TextSpan(
                        if text_sections
                            .last()
                            .is_some_and(|l| Self::space_between(&l.0.0, &f.text))
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
    pub fn as_string(&self) -> String {
        self.as_text_sections()
            .into_iter()
            .map(|text_section| text_section.0.0)
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
