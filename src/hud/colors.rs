use bevy::prelude::{Color, Mix};

// Text colors

/// For important text
pub(crate) const HARD_TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

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

/// Varying from `BAD_TEXT_COLOR` (0.0) over `WARN_TEXT_COLOR` (0.5) to `GOOD_TEXT_COLOR` (1.0)
pub(crate) fn text_color(zero_to_one: f32) -> Color {
    let (part, min_color, max_color) = if 0.5 <= zero_to_one {
        (2.0 * zero_to_one - 1.0, WARN_TEXT_COLOR, GOOD_TEXT_COLOR)
    } else {
        (2.0 * zero_to_one, BAD_TEXT_COLOR, WARN_TEXT_COLOR)
    };
    min_color.mix(&max_color, part)
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn mixing_works() {
        assert_eq!(text_color(0.0), BAD_TEXT_COLOR);
        assert_eq!(text_color(0.5), WARN_TEXT_COLOR);
        assert_eq!(text_color(1.0), GOOD_TEXT_COLOR);
    }
}
