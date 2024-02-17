use crate::prelude::{HorizontalDirection, Timestamp};
use bevy::{
    ecs::entity::EntityHashMap,
    prelude::{Entity, Resource, TextStyle},
};

#[derive(Resource)]
pub(crate) struct InventoryScreen {
    pub(super) panel: Entity,
    pub(super) selected_item: Option<Entity>,
    pub(super) previous_items: EntityHashMap<Entity>,
    pub(super) next_items: EntityHashMap<Entity>,
    pub(super) drop_direction: HorizontalDirection,
    pub(super) section_text_style: TextStyle,
    pub(super) selected_section_text_style: TextStyle,
    pub(super) item_text_style: TextStyle,
    pub(super) selected_item_text_style: TextStyle,
    pub(super) last_time: Timestamp,
}
