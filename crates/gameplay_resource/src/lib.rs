mod plugin;
mod set;
mod systems;

pub use plugin::GameplayResourcePlugin;
pub use set::GampelayResourceSet;

use systems::{add_resource, remove_resource};
