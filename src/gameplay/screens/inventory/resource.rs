use crate::gameplay::screens::inventory::components::{
    InventoryItemDescription, InventoryItemLine,
};
use crate::gameplay::screens::inventory::section::InventorySection;
use crate::gameplay::HorizontalDirection;
use crate::hud::SelectionList;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::{Button, Children, Entity, KeyCode, Query, Resource, Text, TextStyle, With};
use units::Timestamp;

#[derive(Resource)]
pub(super) struct InventoryScreen {
    pub(super) panel: Entity,
    pub(super) selection_list: SelectionList,
    pub(super) drop_direction: HorizontalDirection,
    pub(super) section_by_item: EntityHashMap<InventorySection>,
    pub(super) section_text_style: TextStyle,
    pub(super) drop_section_text_style: TextStyle,
    pub(super) item_text_style: TextStyle,
    pub(super) selected_item_text_style: TextStyle,
    pub(super) last_time: Timestamp,
}

impl InventoryScreen {
    pub(super) fn adjust_selection(
        &mut self,
        item_lines: &Query<(&InventoryItemLine, &Children)>,
        item_texts: &mut Query<(&mut Text, Option<&InventoryItemDescription>)>,
        item_buttons: &Query<&Children, With<Button>>,
        key_code: &KeyCode,
    ) {
        self.highlight_selected(item_lines, item_texts, item_buttons, false);
        self.selection_list.adjust(key_code.into(), key_code.into());
        self.highlight_selected(item_lines, item_texts, item_buttons, true);
    }

    pub(super) fn highlight_selected(
        &self,
        item_lines: &Query<(&InventoryItemLine, &Children)>,
        item_texts: &mut Query<(&mut Text, Option<&InventoryItemDescription>)>,
        item_buttons: &Query<&Children, With<Button>>,
        show_selected: bool,
    ) {
        let Some(selected) = self.selection_list.selected else {
            return;
        };
        let (_, children) = &item_lines
            .get(selected)
            .expect("Highlighted item should ba found");

        let style = if show_selected {
            &self.selected_item_text_style
        } else {
            &self.item_text_style
        };
        for child in *children {
            if let Ok((mut text, description)) = item_texts.get_mut(*child) {
                if let Some(description) = description {
                    text.sections = description.0.as_text_sections(style);
                } else {
                    set_text_style(&mut text, style);
                };
            } else {
                let button_children = item_buttons
                    .get(*child)
                    .expect("Item line child should be a text or a button");
                for button_child in button_children {
                    let (mut text, _) = item_texts
                        .get_mut(*button_child)
                        .expect("Item line buttons should contain text");
                    set_text_style(&mut text, style);
                }
            }
        }
    }

    pub(super) fn selected_item(&self, item_lines: &Query<&InventoryItemLine>) -> Option<Entity> {
        self.selection_list.selected.map(|selected_line| {
            item_lines
                .get(selected_line)
                .expect("Selected row should be found")
                .item
        })
    }
}

fn set_text_style(text: &mut Text, style: &TextStyle) {
    text.sections
        .first_mut()
        .expect("Item texts should have a first text section")
        .style = style.clone();
}
