mod base;
mod character;
mod crafting;
mod death;
mod inventory;
mod menu;
mod plugin;
mod quality;
mod state;
mod tool;
mod transitioning;
mod waiting;

pub(crate) use self::plugin::ScreensPlugin;
pub(crate) use self::state::GameplayScreenState;

use self::base::BaseScreenPlugin;
use self::character::CharacterScreenPlugin;
use self::crafting::CraftingScreenPlugin;
use self::death::DeathScreenPlugin;
use self::inventory::InventoryScreenPlugin;
use self::menu::MenuScreenPlugin;
use self::quality::QualityScreenPlugin;
use self::tool::ToolScreenPlugin;
use self::transitioning::TransitioningScreenPlugin;
use self::waiting::WaitingModalPlugin;
