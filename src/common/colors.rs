use bevy::prelude::Color;

// Text colors

pub(crate) const DEFAULT_TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

/// For unimportant text
pub(crate) const SOFT_TEXT_COLOR: Color = Color::srgb(0.65, 0.65, 0.65);

/// For playing the game
pub(crate) const GOOD_TEXT_COLOR: Color = Color::srgb(0.15, 0.8, 0.15);

/// For aggression and failed user actions
pub(crate) const WARN_TEXT_COLOR: Color = Color::srgb(0.8, 0.8, 0.15);

/// For erros and for quitting
pub(crate) const BAD_TEXT_COLOR: Color = Color::srgb(1.0, 0.31, 0.31);

/// For zombies and stuff they touched
pub(crate) const FILTHY_COLOR: Color = Color::srgb(0.73, 0.4, 1.0);

// Button colors

pub(crate) const DEFAULT_BUTTON_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
pub(crate) const HOVERED_BUTTON_COLOR: Color = Color::srgb(0.25, 0.25, 0.25);

// Panel colors

pub(crate) const PANEL_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.85);
