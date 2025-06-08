mod components;
mod plugin;
mod resource;
mod systems;

pub(crate) use self::components::{Consumed, RecipeSituation};
pub(crate) use self::plugin::CraftingScreenPlugin;

use self::components::{
    AlternativeSituation, ComponentSituation, DetectedQuantity, QualitySituation, ToolSituation,
};
use self::resource::CraftingScreen;
