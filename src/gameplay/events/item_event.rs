use bevy::prelude::{Entity, Event};

#[allow(unused)]
pub(crate) trait ItemChange: Clone + Send + Sync + 'static {}

#[allow(unused)]
#[must_use]
#[derive(Clone, Debug, Event)]
pub(crate) struct ItemEvent<T: ItemChange> {
    pub(crate) item_entity: Entity,
    pub(crate) change: T,
}
