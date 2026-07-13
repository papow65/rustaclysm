use bevy::prelude::{Component, Entity, Vec};

/// Used on a pocket, for the item that has the pocket.
///
/// Required
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = Pockets)]
pub struct PocketOf {
    pub item_entity: Entity,
}

/// Used on an item, for all pockets that the item has.
///
/// Optional, because not all items have pockets
#[derive(Debug, Component)]
#[relationship_target(relationship = PocketOf, linked_spawn)]
pub struct Pockets {
    pocket_entities: Vec<Entity>,
}

impl Pockets {
    #[must_use]
    pub fn pocket_entities(&self) -> &[Entity] {
        &self.pocket_entities
    }
}

/// Used on an item, for the pocket that contains the item.
///
/// Optional, because not all items are inside a pocket
#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[relationship(relationship_target = PocketContents)]
pub struct InPocket {
    pub pocket_entity: Entity,
}

/// Used on a pocket, for the items that the pocket contains.
///
/// Optional, because pockets may be empty
#[derive(Debug, Component)]
#[relationship_target(relationship = InPocket, linked_spawn)]
pub struct PocketContents {
    item_entities: Vec<Entity>,
}

impl PocketContents {
    pub(crate) fn item_entities(&self) -> &[Entity] {
        &self.item_entities
    }
}

/// Used on an item, for the entity that is wielding it.
///
/// Optional, because not all items are wielded
#[derive(Clone, Copy, Debug, PartialEq, Component)]
#[relationship(relationship_target = WieldedItems)]
pub struct WieldedBy {
    pub wielder_entity: Entity,
}

/// Used on an entity (typically an actor), for the items that they are wielding.
///
/// Optional, because not all actors wield items
#[derive(Debug, Component)]
#[relationship_target(relationship = WieldedBy, linked_spawn)]
pub struct WieldedItems {
    item_entities: Vec<Entity>,
}

impl WieldedItems {
    #[must_use]
    pub fn item_entities(&self) -> &[Entity] {
        &self.item_entities
    }
}
