use crate::gameplay::screens::crafting::{components::RecipeSituation, systems::StartCraftSystem};
use bevy::prelude::{Entity, Query, Resource, TextColor};
use hud::{SelectionList, SelectionListStep};
use units::Timestamp;

#[derive(Resource)]
pub(super) struct CraftingScreen {
    pub(super) recipe_list: Entity,
    pub(super) selection_list: SelectionList,
    pub(super) recipe_details: Entity,
    pub(super) last_time: Timestamp,
    start_craft_system: StartCraftSystem,
}

impl CraftingScreen {
    pub(crate) fn new(
        recipe_list: Entity,
        selection_list: SelectionList,
        recipe_details: Entity,
        last_time: Timestamp,
        start_craft_system: StartCraftSystem,
    ) -> Self {
        Self {
            recipe_list,
            selection_list,
            recipe_details,
            last_time,
            start_craft_system,
        }
    }

    pub(super) fn adjust_selection(
        &mut self,
        recipes: &mut Query<(&mut TextColor, &RecipeSituation)>,
        step: SelectionListStep,
    ) {
        self.highlight_selected(recipes, false);
        self.selection_list.adjust(step);
        self.highlight_selected(recipes, true);
    }

    pub(super) fn highlight_selected(
        &self,
        recipes: &mut Query<(&mut TextColor, &RecipeSituation)>,
        show_selected: bool,
    ) {
        let Some(selected) = self.selection_list.selected else {
            return;
        };
        let (text_color, recipe) = &mut recipes
            .get_mut(selected)
            .expect("Highlighted recipe should ba found");
        **text_color = recipe.color(show_selected);
    }

    pub(super) fn start_craft_system(&self) -> &StartCraftSystem {
        &self.start_craft_system
    }
}
