use super::section::InventorySection;
use crate::prelude::{HorizontalDirection, SelectionList, Timestamp};
use bevy::{
    ecs::entity::EntityHashMap,
    prelude::{Entity, KeyCode, Resource, TextStyle},
};

#[derive(Resource)]
pub(super) struct InventoryScreen {
    pub(super) panel: Entity,
    pub(super) selection_list: SelectionList,
    pub(super) selected_row: Option<Entity>,
    pub(super) drop_direction: HorizontalDirection,
    pub(super) section_by_item: EntityHashMap<InventorySection>,
    pub(super) section_text_style: TextStyle,
    pub(super) drop_section_text_style: TextStyle,
    pub(super) item_text_style: TextStyle,
    pub(super) selected_item_text_style: TextStyle,
    pub(super) last_time: Timestamp,
}

impl InventoryScreen {
    pub(super) fn adjust_selection(&mut self, key_code: &KeyCode) {
        if self.selection_list.adjust(key_code.into(), key_code.into()) {
            self.last_time = Timestamp::ZERO;
        }
    }
}
