pub(crate) use bevy::prelude::Color;
use std::cmp::Eq;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Fragment {
    pub(crate) text: String,
    pub(crate) color: Color,
}

impl Fragment {
    pub(crate) fn new<S>(text: S, color: Color) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            color,
        }
    }
}

// The floats in color are unimportant and often come from constants
impl Eq for Fragment {}
