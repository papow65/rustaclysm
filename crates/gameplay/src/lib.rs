mod plugin;
mod screens;
mod systems;

pub use self::plugin::GameplayPlugin;

use self::screens::ScreensPlugin;
use self::systems::{
    check_failed_asset_loading, count_assets, count_pos, create_gameplay_key_bindings,
};
