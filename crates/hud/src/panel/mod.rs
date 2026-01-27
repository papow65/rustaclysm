mod colors;
mod margins;
mod modal;
mod plugin;
mod screen;
mod systems;
mod util;

pub use self::colors::{DEFAULT_SCROLLBAR_COLOR, HOVERED_SCROLLBAR_COLOR, PANEL_COLOR};
pub use self::modal::spawn_modal_panel;
pub use self::screen::{
    LargeNode, scroll_panel, scroll_panel_with_content_entity, scroll_screen, spawn_panel_root,
};
pub use self::util::max_scroll;

pub(crate) use self::plugin::PanelPlugin;

use self::margins::{SCREEN_MARGINS, SMALL_PADDING};
