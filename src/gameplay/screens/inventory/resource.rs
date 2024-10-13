use crate::gameplay::screens::inventory::components::{
    InventoryItemDescription, InventoryItemLine,
};
use crate::gameplay::screens::inventory::section::InventorySection;
use crate::gameplay::HorizontalDirection;
use crate::hud::SelectionList;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::{
    BuildChildren, Button, ChildBuild, Children, Commands, DespawnRecursiveExt, Entity, KeyCode,
    Query, Resource, TextStyle, With,
};
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
        commands: &mut Commands,
        item_lines: &Query<(&InventoryItemLine, &Children)>,
        item_texts: &Query<(Entity, &InventoryItemDescription)>,
        item_buttons: &Query<&Children, With<Button>>,
        text_styles: &mut Query<&mut TextStyle>,
        key_code: &KeyCode,
    ) {
        self.highlight_selected(
            commands,
            item_lines,
            item_texts,
            item_buttons,
            text_styles,
            false,
        );
        self.selection_list.adjust(key_code.into(), key_code.into());
        self.highlight_selected(
            commands,
            item_lines,
            item_texts,
            item_buttons,
            text_styles,
            true,
        );
    }

    pub(super) fn highlight_selected(
        &self,
        commands: &mut Commands,
        item_lines: &Query<(&InventoryItemLine, &Children)>,
        item_texts: &Query<(Entity, &InventoryItemDescription)>,
        item_buttons: &Query<&Children, With<Button>>,
        text_styles: &mut Query<&mut TextStyle>,
        show_selected: bool,
    ) {
        let Some(selected) = self.selection_list.selected else {
            return;
        };
        let (_, children) = &item_lines
            .get(selected)
            .expect("Highlighted item should ba found");

        let used_text_style = if show_selected {
            &self.selected_item_text_style
        } else {
            &self.item_text_style
        };
        for child in *children {
            if let Ok((entity, description)) = item_texts.get(*child) {
                // Item name
                commands
                    .entity(entity)
                    .despawn_descendants()
                    .with_children(|parent| {
                        for section in description.0.as_text_sections(used_text_style) {
                            parent.spawn(section);
                        }
                    });
            } else if let Ok(mut text_style) = text_styles.get_mut(*child) {
                // Item weight, and size
                *text_style = used_text_style.clone();
            } else if let Ok(button_children) = item_buttons.get(*child) {
                // Item buttons
                for button_child in button_children {
                    let mut text_style = text_styles
                        .get_mut(*button_child)
                        .expect("Item line buttons should contain text");
                    *text_style = used_text_style.clone();
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
