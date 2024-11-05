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

    pub(crate) fn walk<W: ItemHierarchyWalker>(
        &self,
        walker: &W,
        item_entity: Entity,
        level: usize,
    ) -> W::Output {
        let pockets_output = self.pockets_in(item_entity).map(|(pocket_entity, pocket)| {
            let contents_output = self
                .items_in(pocket_entity)
                .map(|content| self.walk(walker, content.entity, level + 1));
            walker.visit_pocket(pocket, contents_output, level)
        });
        walker.visit_item(
            self.items.get(item_entity).expect("Valid item expected"),
            pockets_output,
            level,
        )
    }
}

pub(crate) trait ItemHierarchyWalker {
    type Output;

    fn visit_item(
        &self,
        item: ItemItem,
        pockets_output: impl Iterator<Item = Self::Output>,
        level: usize,
    ) -> Self::Output;

    fn visit_pocket(
        &self,
        pocket: &Pocket,
        contents_output: impl Iterator<Item = Self::Output>,
        level: usize,
    ) -> Self::Output;
}
