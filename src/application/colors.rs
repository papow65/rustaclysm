use bevy::prelude::Color;

// Text colors

pub(crate) const DEFAULT_TEXT_COLOR: Color = Color::rgb(0.85, 0.85, 0.85);

/** For unimportant text */
pub(crate) const SOFT_TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

/** For playing the game */
pub(crate) const GOOD_TEXT_COLOR: Color = Color::rgb(0.35, 0.85, 0.35);

/** For aggression and failed user actions */
pub(crate) const WARN_TEXT_COLOR: Color = Color::rgb(0.85, 0.85, 0.35);

/** For erros and for quitting */
pub(crate) const BAD_TEXT_COLOR: Color = Color::rgb(0.85, 0.35, 0.35);

// Button colors

pub(crate) const DEFAULT_BUTTON_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
pub(crate) const HOVERED_BUTTON_COLOR: Color = Color::rgb(0.25, 0.25, 0.25);

// Panel colors

pub(crate) const PANEL_COLOR: Color = Color::rgba(0.25, 0.25, 0.25, 0.6);
