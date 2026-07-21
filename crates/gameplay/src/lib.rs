mod fixed;
mod input;
mod plugin;

pub use self::plugin::GameplayPlugin;

use self::fixed::{check_failed_asset_loading, count_assets, count_pos};
use self::input::create_gameplay_key_bindings;
