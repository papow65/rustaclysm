use crate::gameplay::screens::inventory::components::{InventoryItemDescription, InventoryItemRow};
use crate::gameplay::screens::inventory::section::InventorySection;
use crate::gameplay::HorizontalDirection;
use crate::hud::{Fonts, SelectionList, GOOD_TEXT_COLOR, HARD_TEXT_COLOR};
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::{
    BuildChildren, Button, ChildBuild, Children, Commands, DespawnRecursiveExt, Entity, KeyCode,
    Query, Resource, TextColor, With,
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
        commands: &mut Commands,
        fonts: &Fonts,
        item_rows: &Query<(&InventoryItemRow, &Children)>,
        item_texts: &Query<(Entity, &InventoryItemDescription)>,
        item_buttons: &Query<&Children, With<Button>>,
        text_styles: &mut Query<&mut TextColor>,
        key_code: &KeyCode,
    ) {
        self.highlight_selected(
            commands,
            fonts,
            item_rows,
            item_texts,
            item_buttons,
            text_styles,
            false,
        );
        self.selection_list.adjust(key_code.into(), key_code.into());
        self.highlight_selected(
            commands,
            fonts,
            item_rows,
            item_texts,
            item_buttons,
            text_styles,
            true,
        );
    }

    pub(super) fn highlight_selected(
        &self,
        commands: &mut Commands,
        fonts: &Fonts,
        item_rows: &Query<(&InventoryItemRow, &Children)>,
        item_texts: &Query<(Entity, &InventoryItemDescription)>,
        item_buttons: &Query<&Children, With<Button>>,
        text_styles: &mut Query<&mut TextColor>,
        show_selected: bool,
    ) {
        let Some(selected) = self.selection_list.selected else {
            return;
        };
        let (_, children) = &item_rows
            .get(selected)
            .expect("Highlighted item should ba found");

        let used_text_style = if show_selected {
            SELECTED_ITEM_TEXT_COLOR
        } else {
            ITEM_TEXT_COLOR
        };
        for child in *children {
            if let Ok((entity, description)) = item_texts.get(*child) {
                // Item name
                commands
                    .entity(entity)
                    .despawn_descendants()
                    .with_children(|parent| {
                        for section in description
                            .0
                            .as_text_sections(used_text_style, &fonts.regular())
                        {
                            parent.spawn(section);
                        }
                    });
            } else if let Ok(mut text_style) = text_styles.get_mut(*child) {
                // Item weight, and size
                *text_style = used_text_style;
            } else if let Ok(button_children) = item_buttons.get(*child) {
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
