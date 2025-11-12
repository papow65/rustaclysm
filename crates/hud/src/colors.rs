use bevy::prelude::{BackgroundColor, Color, Mix as _, TextColor};

// Text colors

/// For important text
pub const HARD_TEXT_COLOR: TextColor = TextColor(Color::srgb(0.85, 0.85, 0.85));

/// For unimportant text
pub const SOFT_TEXT_COLOR: TextColor = TextColor(Color::srgb(0.65, 0.65, 0.65));

/// For playing the game
pub const GOOD_TEXT_COLOR: TextColor = TextColor(Color::srgb(0.15, 0.8, 0.15));

/// For aggression and nonsensical user input
pub const WARN_TEXT_COLOR: TextColor = TextColor(Color::srgb(0.8, 0.8, 0.15));

/// For errors and for quitting
pub const BAD_TEXT_COLOR: TextColor = TextColor(Color::srgb(1.0, 0.31, 0.31));

/// For zombies and stuff they touched
pub const FILTHY_COLOR: TextColor = TextColor(Color::srgb(0.73, 0.4, 1.0));

/// For liquids
pub const BLUE_TEXT_COLOR: TextColor = TextColor(Color::srgb(0.25, 0.75, 1.0));

pub const DEFAULT_BUTTON_COLOR: BackgroundColor = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
pub const HOVERED_BUTTON_COLOR: BackgroundColor = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));

pub const DEFAULT_SCROLLBAR_COLOR: BackgroundColor = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
pub const HOVERED_SCROLLBAR_COLOR: BackgroundColor = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));

pub const PANEL_COLOR: BackgroundColor = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85));

/// Varying from `BAD_TEXT_COLOR` (0.0) over `WARN_TEXT_COLOR` (0.5) to `GOOD_TEXT_COLOR` (1.0)
///
/// Suited where 1.0 is the normal situation, and for progressing time
#[must_use]
pub fn text_color_expect_full(zero_to_one: f32) -> TextColor {
    text_color_over(zero_to_one, WARN_TEXT_COLOR.0)
}

/// Varying from `BAD_TEXT_COLOR` (0.0) over `HARD_TEXT_COLOR` (0.5) to `GOOD_TEXT_COLOR` (1.0)
///
/// Suited where 0.5 is more common than 0.0, or 1.0
#[must_use]
pub fn text_color_expect_half(zero_to_one: f32) -> TextColor {
    text_color_over(zero_to_one, HARD_TEXT_COLOR.0)
}

/// Varying from `BAD_TEXT_COLOR` (0.0) over the given color (0.5) to `GOOD_TEXT_COLOR` (1.0)
fn text_color_over(zero_to_one: f32, over: Color) -> TextColor {
    let zero_to_one = zero_to_one.clamp(0.0, 1.0);
    let (part, min_color, max_color) = if 0.5 <= zero_to_one {
        (2.0 * zero_to_one - 1.0, over, GOOD_TEXT_COLOR.0)
    } else {
        (2.0 * zero_to_one, BAD_TEXT_COLOR.0, over)
    };
    TextColor(min_color.mix(&max_color, part))
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn mixing_works() {
        assert_eq!(text_color_expect_full(0.0).0, BAD_TEXT_COLOR.0);
        assert_eq!(text_color_expect_full(0.5).0, WARN_TEXT_COLOR.0);
        assert_eq!(text_color_expect_full(1.0).0, GOOD_TEXT_COLOR.0);

        assert_eq!(text_color_expect_half(0.0).0, BAD_TEXT_COLOR.0);
        assert_eq!(text_color_expect_half(0.5).0, HARD_TEXT_COLOR.0);
        assert_eq!(text_color_expect_half(1.0).0, GOOD_TEXT_COLOR.0);
    }
}
