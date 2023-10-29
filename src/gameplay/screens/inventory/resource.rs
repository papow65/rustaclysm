use crate::prelude::{HorizontalDirection, Timestamp};
use bevy::{
    prelude::{Entity, Resource, TextStyle},
    utils::HashMap,
};

#[derive(Resource)]
pub(crate) struct InventoryScreen {
    pub(super) panel: Entity,
    pub(super) selected_item: Option<Entity>,
    pub(super) previous_items: HashMap<Entity, Entity>,
    pub(super) next_items: HashMap<Entity, Entity>,
    pub(super) drop_direction: HorizontalDirection,
    pub(super) section_text_style: TextStyle,
    pub(super) selected_section_text_style: TextStyle,
    pub(super) item_text_style: TextStyle,
    pub(super) selected_item_text_style: TextStyle,
    pub(super) last_time: Timestamp,
}
