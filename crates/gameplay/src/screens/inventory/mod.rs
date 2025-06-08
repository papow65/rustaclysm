mod components;
mod plugin;
mod resource;
mod row_spawner;
mod section;
mod systems;

pub(crate) use self::plugin::InventoryScreenPlugin;

use self::components::InventoryItemRow;
use self::section::InventorySection;
use self::systems::InventorySystem;
