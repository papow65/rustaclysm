use crate::prelude::{HorizontalDirection, SelectionList, Timestamp};
use bevy::prelude::{Entity, Resource, TextStyle};

#[derive(Resource)]
pub(super) struct InventoryScreen {
    pub(super) panel: Entity,
    pub(super) selection_list: SelectionList,
    pub(super) selected_row: Option<Entity>,
    pub(super) drop_direction: HorizontalDirection,
    pub(super) section_text_style: TextStyle,
    pub(super) selected_section_text_style: TextStyle,
    pub(super) item_text_style: TextStyle,
    pub(super) selected_item_text_style: TextStyle,
    pub(super) last_time: Timestamp,
}
