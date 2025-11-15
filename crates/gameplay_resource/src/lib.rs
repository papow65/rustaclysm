mod plugin;
mod set;
mod systems;

pub use plugin::gameplay_resource_plugin;
pub use set::GampelayResourceSet;

use systems::{add_resource, remove_resource};
