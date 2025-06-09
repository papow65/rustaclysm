use bevy::prelude::{Component, Entity, Vec};

/// Used on a pocket, for the item that has the pocket.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = Pockets)]
pub(crate) struct PocketOf {
    pub(crate) item_entity: Entity,
}

/// Used on an item, for all pockets that the item has.
///
/// Optional, because not all items have pockets
#[derive(Debug, Component)]
#[relationship_target(relationship = PocketOf, linked_spawn)]
pub(crate) struct Pockets {
    pocket_entities: Vec<Entity>,
}

impl Pockets {
    pub(crate) fn pocket_entities(&self) -> &[Entity] {
        &self.pocket_entities
    }
}

/// Used on an item, for the pocket that contains the item.
///
/// Optional, because not all items are inside a pocket
#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[relationship(relationship_target = PocketContents)]
pub(crate) struct InPocket {
    pub(crate) pocket_entity: Entity,
}

/// Used on a pocket, for the items that the pocket contains.
///
/// Optional, because pockets may be empty
#[derive(Debug, Component)]
#[relationship_target(relationship = InPocket, linked_spawn)]
pub(crate) struct PocketContents {
    item_entities: Vec<Entity>,
}

impl PocketContents {
    pub(crate) fn item_entities(&self) -> &[Entity] {
        &self.item_entities
    }
}
