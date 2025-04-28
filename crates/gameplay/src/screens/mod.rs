mod base;
mod character;
mod crafting;
mod death;
mod inventory;
mod menu;
mod plugin;
mod state;

pub(crate) use self::base::update_camera_offset;
pub(crate) use self::crafting::{Consumed, RecipeSituation};
pub(crate) use self::plugin::ScreensPlugin;
pub(crate) use self::state::GameplayScreenState;

use self::base::BaseScreenPlugin;
use self::character::CharacterScreenPlugin;
use self::crafting::CraftingScreenPlugin;
use self::death::DeathScreenPlugin;
use self::inventory::InventoryScreenPlugin;
use self::menu::MenuScreenPlugin;
