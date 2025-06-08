use crate::screens::inventory::{InventoryItemRow, InventorySection, InventorySystem};
use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::prelude::{BackgroundColor, Button, Children, Entity, Query, Resource, TextColor, With};
use gameplay_location::HorizontalDirection;
use hud::{GOOD_TEXT_COLOR, HARD_TEXT_COLOR, HOVERED_BUTTON_COLOR};
use units::Timestamp;

pub(super) const ITEM_TEXT_COLOR: TextColor = HARD_TEXT_COLOR;
pub(super) const SELECTED_ITEM_TEXT_COLOR: TextColor = GOOD_TEXT_COLOR;

#[derive(Resource)]
pub(super) struct InventoryScreen {
    pub(super) panel: Entity,
    pub(super) drop_direction: HorizontalDirection,
    pub(super) section_by_item: EntityHashMap<InventorySection>,
    pub(super) last_time: Timestamp,
    pub(super) inventory_system: InventorySystem,
}

impl InventoryScreen {
    pub(crate) const fn new(
        panel: Entity,
        drop_direction: HorizontalDirection,
        section_by_item: EntityHashMap<InventorySection>,
        last_time: Timestamp,
        inventory_system: InventorySystem,
    ) -> Self {
        Self {
            panel,
            drop_direction,
            section_by_item,
            last_time,
            inventory_system,
        }
    }

    pub(super) fn highlight_selected(
        item_row: Entity,
        item_rows: &mut Query<(&InventoryItemRow, &mut BackgroundColor, &Children)>,
        item_buttons: &Query<&Children, With<Button>>,
        text_styles: &mut Query<&mut TextColor>,
        show_selected: bool,
    ) {
        let (_, mut background_color, children) = item_rows
            .get_mut(item_row)
            .expect("Highlighted item row should ba found");

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
}
