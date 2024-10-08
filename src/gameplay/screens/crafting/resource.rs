use crate::gameplay::screens::crafting::components::RecipeSituation;
use crate::hud::SelectionList;
use bevy::prelude::{Entity, KeyCode, Query, Resource, Text};
use units::Timestamp;

#[derive(Resource)]
pub(super) struct CraftingScreen {
    pub(super) recipe_list: Entity,
    pub(super) selection_list: SelectionList,
    pub(super) recipe_details: Entity,
    pub(super) last_time: Timestamp,
}

impl CraftingScreen {
    pub(super) fn adjust_selection(
        &mut self,
        recipes: &mut Query<(&mut Text, &RecipeSituation)>,
        key_code: &KeyCode,
    ) {
        self.highlight_selected(recipes, false);
        self.selection_list.adjust(key_code.into(), key_code.into());
        self.highlight_selected(recipes, true);
    }

    pub(super) fn highlight_selected(
        &self,
        recipes: &mut Query<(&mut Text, &RecipeSituation)>,
        show_selected: bool,
    ) {
        let Some(selected) = self.selection_list.selected else {
            return;
        };
        let (text, recipe) = &mut recipes
            .get_mut(selected)
            .expect("Highlighted recipe should ba found");
        text.sections
            .first_mut()
            .expect("Recipes should have a first text section")
            .style
            .color = recipe.color(show_selected);
    }
}
