use crate::{
    Amount, ContainerLimits, Fragment, Item, ItemItem, Phrase, Pocket, PocketItem, SealedPocket,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Children, Entity, Query};
use cdda_json_files::{PocketInfo, PocketType, UntypedInfoId};
use std::{iter::once, num::NonZeroUsize, sync::Arc};

pub(crate) enum PocketWrapper<'i> {
    Concrete(PocketItem<'i>),
    Lazy(Arc<PocketInfo>),
}

impl PocketWrapper<'_> {
    pub(crate) const fn entity(&self) -> Option<Entity> {
        match self {
            Self::Concrete(pocket) => Some(pocket.entity),
            Self::Lazy(_) => None,
        }
    }

    pub(crate) fn pocket_type(&self) -> PocketType {
        match self {
            Self::Concrete(pocket) => pocket.info.pocket_type,
            Self::Lazy(info) => info.pocket_type,
        }
    }

    pub(crate) fn sealed(&self) -> Option<SealedPocket> {
        match self {
            Self::Concrete(pocket) => pocket.sealed.copied(),
            Self::Lazy(info) => info.sealed_data.as_ref().map(SealedPocket::from),
        }
    }
}

pub(crate) struct Subitems<'i> {
    pocket_wrapper: PocketWrapper<'i>,
    items: Vec<ItemItem<'i>>,
}

pub(crate) struct ShownContents<'i> {
    sealed: Option<SealedPocket>,
    contents: Vec<ItemItem<'i>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct InPocket {
    pub(crate) type_: PocketType,
    pub(crate) single_in_type: bool,
    pub(crate) depth: NonZeroUsize,
}

#[derive(SystemParam)]
pub(crate) struct ItemHierarchy<'w, 's> {
    limits: Query<'w, 's, &'static ContainerLimits>,
    parents: Query<'w, 's, &'static Children>,
    items: Query<'w, 's, Item>,
    pockets: Query<'w, 's, Pocket>,
}

impl<'w> ItemHierarchy<'w, '_> {
    pub(crate) fn exists(&self, item: Entity) -> bool {
        self.items.get(item).is_ok()
    }

    pub(crate) fn items_in(&self, container: Entity) -> impl Iterator<Item = ItemItem> + use<'_> {
        self.parents
            .get(container)
            .into_iter()
            .flat_map(IntoIterator::into_iter)
            .flat_map(|item| self.items.get(*item)) // Filtering out the models
    }

    pub(crate) fn pockets_in(&self, container: &ItemItem) -> Vec<PocketWrapper> {
        let concrete_pockets = container
            .children
            .into_iter()
            .flat_map(IntoIterator::into_iter)
            .copied()
            .flat_map(|pocket| self.pockets.get(pocket)) // Filtering out the models
            .map(PocketWrapper::Concrete)
            .collect::<Vec<_>>();

        if !concrete_pockets.is_empty() {
            return concrete_pockets;
        }

        container
            .common_info
            .pocket_data
            .clone()
            .into_iter()
            .flat_map(move |pocket_infos| pocket_infos.into_iter().map(PocketWrapper::Lazy))
            .collect()
    }

    pub(crate) fn container(&self, container_entity: Entity) -> &ContainerLimits {
        self.limits
            .get(container_entity)
            .expect("An existing container")
    }

    pub(crate) fn walk(
        &self,
        handler: &mut impl ItemHandler,
        items: impl IntoIterator<Item = ItemItem<'w>>,
    ) {
        for item in items {
            self.walk_item(handler, None, &item);
        }
    }

    fn walk_item(
        &self,
        handler: &mut impl ItemHandler,
        in_pocket: Option<InPocket>,
        item: &ItemItem,
    ) {
        // TODO make sure all pockets are present on containers

        let depth = in_pocket.map_or(NonZeroUsize::MIN, |ip| ip.depth.saturating_add(1));
        let prefix = in_pocket.and_then(|ip| prefix(ip.type_, depth.get(), ip.single_in_type));
        let suffix = in_pocket.map_or_else(
            || Some(Fragment::soft("\n")),
            |ip| suffix(ip.type_, ip.single_in_type),
        );

        let mut shown_contents = self.shown_contents(item);
        let item_fragments = self.item_fragments(prefix, suffix, item, shown_contents.as_mut());
        handler.handle_item(item, item_fragments);

        if let Some(shown_contents) = shown_contents {
            if !shown_contents.contents.is_empty() {
                let in_pocket = Some(InPocket {
                    type_: PocketType::Container,
                    single_in_type: shown_contents.contents.len() == 1,
                    depth,
                });
                for subitem in shown_contents.contents {
                    self.walk_item(handler, in_pocket, &subitem);
                }
            }
        }

        if handler.show_other_pockets() {
            for pocket_type in [
                PocketType::Mod,
                PocketType::Corpse,
                PocketType::Software,
                PocketType::Ebook,
                PocketType::Migration,
                PocketType::Last,
            ] {
                let items = self
                    .pockets(item, pocket_type)
                    .flat_map(|subitems| subitems.items)
                    .collect::<Vec<_>>();
                let in_pocket = Some(InPocket {
                    type_: pocket_type,
                    single_in_type: items.len() == 1,
                    depth,
                });
                for subitem in items {
                    self.walk_item(handler, in_pocket, &subitem);
                }
            }
        }
    }

    fn shown_contents(&self, item: &ItemItem) -> Option<ShownContents> {
        let contents = self
            .pockets(item, PocketType::Container)
            .collect::<Vec<_>>();
        let first_pocket_sealing = contents
            .first()
            .map(|first_subitems| first_subitems.pocket_wrapper.sealed());
        first_pocket_sealing.map(move |sealed| ShownContents {
            sealed,
            contents: contents
                .into_iter()
                .flat_map(|subitems| subitems.items)
                .collect::<Vec<_>>(),
        })
    }

    fn item_fragments(
        &self,
        prefix: Option<Fragment>,
        suffix: Option<Fragment>,
        item: &ItemItem,
        shown_contents: Option<&mut ShownContents>,
    ) -> Vec<Fragment> {
        let magazine_wells = self.pockets(item, PocketType::MagazineWell);

        let mut magazine_pockets = self.pockets(item, PocketType::Magazine).collect::<Vec<_>>();

        let phrase = Phrase::from_fragments(prefix.into_iter().collect())
            .extend(item.fragments())
            .debug(format!("[{}]", item.common_info.id.fallback_name()))
            .extend(Self::battery_charge_fragments(item, &mut magazine_pockets))
            .extend(self.magazine_fragments(magazine_wells, magazine_pockets))
            .extend(Self::item_tag_fragments(item, &shown_contents));

        if let Some(shown_contents) = shown_contents {
            if shown_contents.contents.is_empty() {
                phrase
            } else if self.is_single_hierarchy(&shown_contents.contents) {
                phrase.push(Fragment::good(">")).extend(
                    shown_contents
                        .contents
                        .drain(0..=0)
                        .flat_map(|subitem| self.inline_item_fragments(&subitem)),
                )
            } else {
                phrase.push(Fragment::good(format!(
                    "> {}+",
                    shown_contents.contents.len()
                )))
            }
        } else {
            phrase
        }
        .extend(suffix)
        .fragments
    }

    fn battery_charge_fragments(
        item: &ItemItem<'_>,
        magazine_pockets: &mut Vec<Subitems>,
    ) -> impl Iterator<Item = Fragment> + use<> {
        item.magazine_info
            .filter(|magazine| {
                magazine
                    .ammo_type
                    .0
                    .contains(&UntypedInfoId::new("battery"))
            })
            .and_then(|magazine| magazine.capacity)
            .map(|capacity| {
                let mut battery_amounts = magazine_pockets
                    .drain(..)
                    .flat_map(|subitems| subitems.items)
                    .map(|item| item.amount)
                    .collect::<Vec<_>>();

                assert!(
                    battery_amounts.len() <= 1,
                    "Too many battery charges: {battery_amounts:?}"
                );

                [
                    Fragment::soft("at"),
                    battery_amounts
                        .pop()
                        .unwrap_or(&Amount::ZERO)
                        .fragment_in_range(capacity),
                    Fragment::soft(format!("/{capacity}")),
                ]
            })
            .into_iter()
            .flatten()
    }

    fn magazine_fragments<'a>(
        &self,
        magazine_wells: impl Iterator<Item = Subitems<'a>>,
        magazine_pockets: Vec<Subitems<'a>>,
    ) -> Vec<Fragment> {
        let mut magazine_wells = magazine_wells.peekable();
        let unloaded = magazine_wells
            .peek()
            .is_some_and(|info| info.items.is_empty());
        let mut fragments = if unloaded {
            vec![
                Fragment::soft("("),
                Fragment::bad("not loaded"),
                Fragment::soft(")"),
            ]
        } else {
            Vec::new()
        };

        let loaded_fragments = magazine_wells
            .chain(magazine_pockets)
            .flat_map(|subitems| subitems.items)
            .map(|subitem| self.inline_item_fragments(&subitem))
            .collect::<Vec<_>>()
            .join(&Fragment::soft(", "));
        if !loaded_fragments.is_empty() {
            fragments.push(Fragment::soft("with"));
            fragments.extend(loaded_fragments);
        }

        fragments
    }

    fn item_tag_fragments(
        item: &ItemItem<'_>,
        shown_contents: &Option<&mut ShownContents<'_>>,
    ) -> Vec<Fragment> {
        let mut tags = Vec::new();
        if let Some(ref shown_contents) = *shown_contents {
            if shown_contents.contents.is_empty() {
                tags.push([Fragment::soft("empty")]);
            }
            tags.extend(
                shown_contents
                    .sealed
                    .map(SealedPocket::suffix)
                    .map(|tag| [tag]),
            );
        }
        tags.extend(item.phase.suffix().map(|tag| [tag]));

        if tags.is_empty() {
            Vec::new()
        } else {
            once(Fragment::soft("("))
                .chain(tags.join(&Fragment::soft(",")))
                .chain(once(Fragment::soft(")")))
                .collect()
        }
    }

    fn inline_item_fragments(&self, item: &ItemItem) -> Vec<Fragment> {
        let mut container_data = self.shown_contents(item);
        if let Some(ref container_data) = container_data {
            assert!(
                container_data.contents.len() <= 1,
                "Too many (>1) contents to inline for {item:?}: {:?}",
                &container_data.contents
            );
        }
        self.item_fragments(None, None, item, container_data.as_mut())
    }

    /// At most one subitem, with at most one subitem, with at most one subitem, ...
    fn is_single_hierarchy(&self, items: &[ItemItem]) -> bool {
        match items {
            [] => true,
            [item] => {
                let subitems = self
                    .pockets(item, PocketType::Container)
                    .flat_map(|pocket| pocket.items)
                    .collect::<Vec<_>>();
                self.is_single_hierarchy(&subitems)
            }
            _ => false,
        }
    }

    fn pockets(
        &self,
        item: &ItemItem,
        pocket_type: PocketType,
    ) -> impl Iterator<Item = Subitems<'_>> + use<'_> {
        self.pockets_in(item)
            .into_iter()
            .filter(move |pocket_wrapper| pocket_wrapper.pocket_type() == pocket_type)
            .map(move |pocket_wrapper| {
                // TODO Use default contents when None
                let items = pocket_wrapper
                    .entity()
                    .into_iter()
                    .flat_map(|entity| self.items_in(entity))
                    .collect::<Vec<_>>();

                Subitems {
                    pocket_wrapper,
                    items,
                }
            })
    }
}

fn prefix(pocket_type: PocketType, depth: usize, single_in_type: bool) -> Option<Fragment> {
    let indicator = match pocket_type {
        PocketType::Container => {
            if single_in_type {
                return None;
            } else {
                '>'
            }
        }
        PocketType::Magazine => {
            return Some(Fragment::soft(String::from("with")));
        }
        PocketType::MagazineWell => {
            return Some(Fragment::soft(String::from("(")));
        }
        PocketType::Mod => '+',
        PocketType::Corpse => '_',
        PocketType::Software => 'S',
        PocketType::Ebook => 'E',
        PocketType::Migration => 'M',
        PocketType::Last => '9',
    };
    Some(Fragment::soft(format!(
        "{}'-{indicator}",
        "    ".repeat(depth - 1)
    )))
}

fn suffix(pocket_type: PocketType, single_in_type: bool) -> Option<Fragment> {
    match (pocket_type, single_in_type) {
        (PocketType::Magazine, _) | (PocketType::Container, true) => None,
        (PocketType::MagazineWell, _) => Some(Fragment::soft(")")),
        _ => Some(Fragment::soft("\n")),
    }
}

pub(crate) trait ItemHandler {
    fn handle_item(&mut self, item: &ItemItem, item_fragments: Vec<Fragment>);

    /** Show pockets that are not containers, magazines, or magazine wells */
    fn show_other_pockets(&self) -> bool;
}
