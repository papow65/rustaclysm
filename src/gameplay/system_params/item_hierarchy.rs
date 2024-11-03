use crate::gameplay::item::Pocket;
use crate::gameplay::{ContainerLimits, Item, ItemItem};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Children, Entity, HierarchyQueryExt, Query};

#[derive(SystemParam)]
pub(crate) struct ItemHierarchy<'w, 's> {
    limits: Query<'w, 's, &'static ContainerLimits>,
    children: Query<'w, 's, &'static Children>,
    items: Query<'w, 's, Item>,
    pockets: Query<'w, 's, (Entity, &'static Pocket)>,
}

impl<'w, 's> ItemHierarchy<'w, 's> {
    pub(crate) fn items_in(&self, container: Entity) -> impl Iterator<Item = ItemItem> + use<'_> {
        self.children
            .children(container)
            .iter()
            .flat_map(|&item| self.items.get(item))
    }

    pub(crate) fn pockets_in(
        &self,
        container: Entity,
    ) -> impl Iterator<Item = (Entity, &Pocket)> + use<'_> {
        self.children
            .children(container)
            .iter()
            .flat_map(|&pocket| self.pockets.get(pocket))
    }

    pub(crate) fn container(&self, container_entity: Entity) -> &ContainerLimits {
        self.limits
            .get(container_entity)
            .expect("An existing container")
    }
}
