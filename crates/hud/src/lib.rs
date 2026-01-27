//! This crate provides the [`HudPlugin`], defaults colors, a default font, a default panel, and spacing defaults

mod button;
mod panel;
mod plugin;
mod spacing;
mod text;

pub use self::button::{
    ButtonBuilder, DEFAULT_BUTTON_COLOR, HOVERED_BUTTON_COLOR, RunButton, manage_button_input,
    trigger_button_action,
};
pub use self::panel::{
    DEFAULT_SCROLLBAR_COLOR, HOVERED_SCROLLBAR_COLOR, LargeNode, PANEL_COLOR, max_scroll,
    scroll_panel, scroll_panel_with_content_entity, scroll_screen, spawn_modal_panel,
    spawn_panel_root,
};
pub use self::plugin::HudPlugin;
pub use self::spacing::{LARGE_SPACING, MEDIUM_SPACING, SMALL_SPACING};
pub use self::text::{
    BAD_TEXT_COLOR, BLUE_TEXT_COLOR, DebugText, DebugTextShown, FILTHY_COLOR, Fonts,
    GOOD_TEXT_COLOR, HARD_TEXT_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR, text_color_expect_full,
    text_color_expect_half, toggle_debug_text,
};
