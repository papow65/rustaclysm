mod colors;
mod components;
mod debug_text_shown;
mod fonts;
mod plugin;
mod systems;

pub use self::colors::{
    BAD_TEXT_COLOR, BLUE_TEXT_COLOR, FILTHY_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR,
    SOFT_TEXT_COLOR, WARN_TEXT_COLOR, text_color_expect_full, text_color_expect_half,
};
pub use self::components::DebugText;
pub use self::debug_text_shown::DebugTextShown;
pub use self::fonts::Fonts;
pub use self::systems::toggle_debug_text;

pub(crate) use self::plugin::TextPlugin;

use self::components::CheckedFont;
use self::systems::{add_missing_font, add_missing_pickable};
