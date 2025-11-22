use crate::screens::crafting::systems::StartCraftSystem;
use bevy::prelude::{Entity, Resource};

#[derive(Resource)]
pub(super) struct CraftingScreen {
    pub(super) recipe_list: Entity,
    pub(super) recipe_details: Entity,
    start_craft_system: StartCraftSystem,
}

impl CraftingScreen {
    pub(crate) const fn new(
        recipe_list: Entity,
        recipe_details: Entity,
        start_craft_system: StartCraftSystem,
    ) -> Self {
        Self {
            recipe_list,
            recipe_details,
            start_craft_system,
        }
    }

    pub(super) const fn start_craft_system(&self) -> &StartCraftSystem {
        &self.start_craft_system
    }
}
