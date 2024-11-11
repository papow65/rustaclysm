use crate::gameplay::screens::inventory::components::InventoryItemRow;
use crate::gameplay::screens::inventory::section::InventorySection;
use crate::gameplay::HorizontalDirection;
use crate::hud::{SelectionList, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, HOVERED_BUTTON_COLOR};
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::{
    BackgroundColor, Button, Children, Entity, KeyCode, Query, Resource, TextColor, With,
};
use units::Timestamp;

pub(super) const ITEM_TEXT_COLOR: TextColor = HARD_TEXT_COLOR;
pub(super) const SELECTED_ITEM_TEXT_COLOR: TextColor = GOOD_TEXT_COLOR;

#[derive(Resource)]
pub(super) struct InventoryScreen {
    pub(super) panel: Entity,
    pub(super) selection_list: SelectionList,
    pub(super) drop_direction: HorizontalDirection,
    pub(super) section_by_item: EntityHashMap<InventorySection>,
    pub(super) last_time: Timestamp,
}

impl InventoryScreen {
    pub(super) fn adjust_selection(
        &mut self,
        item_rows: &mut Query<(&InventoryItemRow, &mut BackgroundColor, &Children)>,
        item_buttons: &Query<&Children, With<Button>>,
        text_styles: &mut Query<&mut TextColor>,
        key_code: &KeyCode,
    ) {
        self.highlight_selected(item_rows, item_buttons, text_styles, false);
        self.selection_list.adjust(key_code.into(), key_code.into());
        self.highlight_selected(item_rows, item_buttons, text_styles, true);
    }

    pub(super) fn highlight_selected(
        &self,
        item_rows: &mut Query<(&InventoryItemRow, &mut BackgroundColor, &Children)>,
        item_buttons: &Query<&Children, With<Button>>,
        text_styles: &mut Query<&mut TextColor>,
        show_selected: bool,
    ) {
        let Some(selected) = self.selection_list.selected else {
            return;
        };
        let (_, mut background_color, children) = item_rows
            .get_mut(selected)
            .expect("Highlighted item should ba found");

        *background_color = if show_selected {
            HOVERED_BUTTON_COLOR
        } else {
            BackgroundColor::DEFAULT
        };

        let used_text_style = if show_selected {
            SELECTED_ITEM_TEXT_COLOR
        } else {
            ITEM_TEXT_COLOR
        };
        for child in children {
            if let Ok(button_children) = item_buttons.get(*child) {
                // Item buttons
                for button_child in button_children {
                    let mut text_style = text_styles
                        .get_mut(*button_child)
                        .expect("Item row buttons should contain text");
                    *text_style = used_text_style;
                }
            }
        }
    }

    pub(super) fn selected_item(&self, item_rows: &Query<&InventoryItemRow>) -> Option<Entity> {
        self.selection_list.selected.map(|selected_row| {
            item_rows
                .get(selected_row)
                .expect("Selected row should be found")
                .item
        })
    }
}
