use std::num::NonZeroUsize;

use crate::gameplay::item::Pocket;
use crate::gameplay::{ContainerLimits, Item, ItemItem};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Children, Entity, HierarchyQueryExt, Query};
use cdda_json_files::PocketType;

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
        in_pocket: Option<InPocket>,
        item_entity: Entity,
    ) -> W::Output {
        const CONTAINERS: &[PocketType] = &[PocketType::Container];
        const MAGAZINES: &[PocketType] = &[PocketType::Magazine];
        const MAGAZINE_WELLS: &[PocketType] = &[PocketType::MagazineWell];
        const OTHER_POCKETS: &[PocketType] = &[
            PocketType::Mod,
            PocketType::Corpse,
            PocketType::Software,
            PocketType::Ebook,
            PocketType::Migration,
            PocketType::Last,
        ];

        let depth = in_pocket.map_or(NonZeroUsize::MIN, |ip| ip.depth.saturating_add(1));
        let contents = self.pockets(walker, item_entity, CONTAINERS, depth);
        let magazines = self.pockets(walker, item_entity, MAGAZINES, depth);
        let magazine_wells = self.pockets(walker, item_entity, MAGAZINE_WELLS, depth);
        let other_pockets = self.pockets(walker, item_entity, OTHER_POCKETS, depth);

        walker.visit_item(
            self.items.get(item_entity).expect("Valid item expected"),
            contents,
            magazines,
            magazine_wells,
            other_pockets,
            in_pocket,
        )
    }

    fn pockets<'a, W: ItemHierarchyWalker>(
        &'a self,
        walker: &'a W,
        item_entity: Entity,
        pocket_types: &'static [PocketType],
        depth: NonZeroUsize,
    ) -> impl Iterator<Item = Subitems<'a, W::Output>> + use<'a, W> {
        let pockets = self
            .pockets_in(item_entity)
            .filter(|(_, p)| pocket_types.contains(&p.type_))
            .map(move |(pocket_entity, pocket)| {
                (pocket, self.items_in(pocket_entity).collect::<Vec<_>>())
            })
            .collect::<Vec<_>>();
        let single_in_type = pockets
            .iter()
            .map(|(_, subitems)| subitems.len())
            .sum::<usize>()
            == 1;
        pockets.into_iter().map(move |(pocket, subitems)| {
            let contents_output = subitems.iter().map(|content| {
                self.walk(
                    walker,
                    Some(InPocket {
                        type_: pocket.type_,
                        single_in_type,
                        depth,
                    }),
                    content.entity,
                )
            });
            Subitems {
                pocket,
                direct_items: subitems.len(),
                output: walker.visit_pocket(contents_output),
            }
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct InPocket {
    pub(crate) type_: PocketType,
    pub(crate) single_in_type: bool,
    pub(crate) depth: NonZeroUsize,
}

pub(crate) struct Subitems<'p, T> {
    pub(crate) pocket: &'p Pocket,
    pub(crate) direct_items: usize,
    pub(crate) output: T,
}

pub(crate) trait ItemHierarchyWalker {
    type Output;

    fn visit_item<'p>(
        &'p self,
        item: ItemItem,
        contents: impl Iterator<Item = Subitems<'p, Self::Output>>,
        magazines: impl Iterator<Item = Subitems<'p, Self::Output>>,
        magazine_wells: impl Iterator<Item = Subitems<'p, Self::Output>>,
        other_pockets: impl Iterator<Item = Subitems<'p, Self::Output>>,
        in_pocket: Option<InPocket>,
    ) -> Self::Output;

    fn visit_pocket(&self, contents_output: impl Iterator<Item = Self::Output>) -> Self::Output;
}
