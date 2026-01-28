use bevy::prelude::TextColor;
use gameplay_location::Pos;
use hud::{
    BAD_TEXT_COLOR, FILTHY_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, SOFT_TEXT_COLOR,
    WARN_TEXT_COLOR,
};
use std::cmp::Eq;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Positioning {
    Pos(Pos),
    Player,
    #[default]
    None,
}

#[must_use]
#[derive(Clone, Debug, Default)]
pub struct Fragment {
    pub text: String,
    pub color: TextColor,
    pub positioning: Positioning,
    pub debug: bool,
}

impl Fragment {
    pub fn you() -> Self {
        Self {
            text: String::from("You"),
            color: GOOD_TEXT_COLOR,
            positioning: Positioning::Player,
            debug: false,
        }
    }

    pub fn soft<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, SOFT_TEXT_COLOR)
    }

    pub fn hard<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, HARD_TEXT_COLOR)
    }

    pub fn good<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, GOOD_TEXT_COLOR)
    }

    pub fn bad<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, BAD_TEXT_COLOR)
    }

    pub fn filthy<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, FILTHY_COLOR)
    }

    pub fn warn<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::colorized(text, WARN_TEXT_COLOR)
    }

    pub fn colorized<S>(text: S, color: TextColor) -> Self
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

    pub const fn positioned(mut self, pos: Pos) -> Self {
        self.positioning = Positioning::Pos(pos);
        self
    }

    pub const fn debug(mut self) -> Self {
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
