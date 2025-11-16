//! This crate provides the [`HudPlugin`], defaults colors, a default font, a default panel, and spacing defaults

mod button;
mod colors;
mod fonts;
mod panel;
mod plugin;
mod selection_list;
mod spacing;

pub use self::button::{ButtonBuilder, RunButton, manage_button_input, trigger_button_action};
pub use self::colors::{
    BAD_TEXT_COLOR, BLUE_TEXT_COLOR, DEFAULT_BUTTON_COLOR, DEFAULT_SCROLLBAR_COLOR, FILTHY_COLOR,
    GOOD_TEXT_COLOR, HARD_TEXT_COLOR, HOVERED_BUTTON_COLOR, HOVERED_SCROLLBAR_COLOR, PANEL_COLOR,
    SOFT_TEXT_COLOR, WARN_TEXT_COLOR, text_color_expect_full, text_color_expect_half,
};
pub use self::fonts::Fonts;
pub use self::panel::{LargeNode, scroll_panel, scroll_screen, selection_list_detail_screen};
pub use self::plugin::HudPlugin;
pub use self::selection_list::{SelectionList, SelectionListStep, scroll_to_selection};
pub use self::spacing::{LARGE_SPACING, MEDIUM_SPACING, SMALL_SPACING};
