//! This crate provides the [`HudPlugin`], defaults colors, a default font, a default panel, and spacing defaults

mod button_builder;
mod colors;
mod fonts;
mod panel_node;
mod plugin;
mod selection_list;
mod spacing;
mod systems;

pub use self::button_builder::{ButtonBuilder, RunButton};
pub use self::colors::{
    BAD_TEXT_COLOR, DEFAULT_BUTTON_COLOR, FILTHY_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR,
    HOVERED_BUTTON_COLOR, PANEL_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR, text_color_expect_full,
    text_color_expect_half,
};
pub use self::fonts::Fonts;
pub use self::panel_node::panel_node;
pub use self::plugin::HudPlugin;
pub use self::selection_list::{SelectionList, SelectionListStep};
pub use self::spacing::{LARGE_SPACING, MEDIUM_SPACING, SMALL_SPACING};
pub use self::systems::{manage_button_input, scroll_to_selection, trigger_button_action};
