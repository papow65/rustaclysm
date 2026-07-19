mod plugin;
mod screens;
mod sidebar;
mod systems;

pub use self::plugin::GameplayPlugin;

use self::screens::{GameplayScreenState, ScreensPlugin};
use self::sidebar::SidebarPlugin;
use self::systems::{
    check_failed_asset_loading, count_assets, count_pos, create_gameplay_key_bindings,
};
