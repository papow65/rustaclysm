//! This module provides the [`HudPlugin`], defaults colors, a default font, a default panel, and spacing defaults

mod button_builder;
mod colors;
mod default_panel;
mod fonts;
mod plugin;
mod scrolling_list;
mod selection_list;
mod spacing;
mod systems;

pub(crate) use self::button_builder::{ButtonBuilder, RunButton};
pub(crate) use self::colors::{
    text_color_expect_full, text_color_expect_half, BAD_TEXT_COLOR, DEFAULT_BUTTON_COLOR,
    FILTHY_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, HOVERED_BUTTON_COLOR, PANEL_COLOR,
    SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
pub(crate) use self::default_panel::DefaultPanel;
pub(crate) use self::fonts::Fonts;
pub(crate) use self::plugin::HudPlugin;
pub(crate) use self::scrolling_list::ScrollingList;
pub(crate) use self::selection_list::{SelectionList, StepDirection, StepSize};
pub(crate) use self::spacing::{LARGE_SPACING, MEDIUM_SPACING, SMALL_SPACING};
pub(crate) use self::systems::{manage_button_input, trigger_button_action};
