use crate::prelude::HorizontalDirection;
use bevy::prelude::{Entity, Resource, TextStyle};

#[derive(Resource)]
pub(crate) struct InventoryScreen {
    pub(super) root: Entity,
    pub(super) section_text_style: TextStyle,
    pub(super) item_text_style: TextStyle,
    pub(super) drop_direction: HorizontalDirection,
}
