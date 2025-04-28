mod base;
mod character;
mod crafting;
mod death;
mod inventory;
mod loading;
mod menu;
mod plugin;
mod state;
mod unloading;

pub use self::state::GameplayScreenState;

pub(crate) use self::base::update_camera_offset;
pub(crate) use self::crafting::{Consumed, RecipeSituation};
pub(crate) use self::plugin::ScreensPlugin;

use self::base::BaseScreenPlugin;
use self::character::CharacterScreenPlugin;
use self::crafting::CraftingScreenPlugin;
use self::death::DeathScreenPlugin;
use self::inventory::InventoryScreenPlugin;
use self::loading::LoadingScreenPlugin;
use self::menu::MenuScreenPlugin;
use self::unloading::UnloadingScreenPlugin;
