use crate::{ContainerLimits, Fragment, Item, ItemItem, Phrase, Pocket, PocketSealing};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Children, Entity, Query, error};
use cdda_json_files::{PocketType, UntypedInfoId};
use std::panic::Location;
use std::{iter::once, num::NonZeroUsize};

#[derive(Clone, Copy, Debug)]
pub(crate) struct InPocket {
    pub(crate) type_: PocketType,
    pub(crate) single_in_type: bool,
    pub(crate) depth: NonZeroUsize,
}

pub(crate) struct Subitems<'p> {
    pocket: &'p Pocket,
    items: Vec<ItemItem<'p>>,
}

struct ContainerData<'a> {
    sealing: PocketSealing,
    contents: Vec<ItemItem<'a>>,
}

#[derive(SystemParam)]
pub(crate) struct ItemHierarchy<'w, 's> {
    limits: Query<'w, 's, &'static ContainerLimits>,
    children: Query<'w, 's, &'static Children>,
    items: Query<'w, 's, Item>,
    pockets: Query<'w, 's, (Entity, &'static Pocket)>,
}

impl<'w> ItemHierarchy<'w, '_> {
    pub(crate) fn exists(&self, item: Entity) -> bool {
        self.items.get(item).is_ok()
    }

    pub(crate) fn items_in(&self, container: Entity) -> impl Iterator<Item = ItemItem> + use<'_> {
        self.children
            .get(container)
            .into_iter()
            .flat_map(IntoIterator::into_iter)
            .flat_map(|item| self.items.get(*item)) // Filtering out the models
    }

    pub(crate) fn pockets_in(
        &self,
        container: Entity,
    ) -> impl Iterator<Item = (Entity, &Pocket)> + use<'_> {
        self.children
            .get(container)
            .inspect_err(|error| error!("{} {error:?}", Location::caller()))
            .into_iter()
            .flat_map(IntoIterator::into_iter)
            .flat_map(|&pocket| self.pockets.get(pocket)) // Filtering out the models
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

        let mut container_data = self.container_data(item);
        let item_fragments = self.item_fragments(prefix, suffix, item, container_data.as_mut());
        handler.handle_item(item, item_fragments);

        if let Some(container_data) = container_data {
            if !container_data.contents.is_empty() {
                let in_pocket = Some(InPocket {
                    type_: PocketType::Container,
                    single_in_type: container_data.contents.len() == 1,
                    depth,
                });
                for subitem in container_data.contents {
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
                    .pockets(item.entity, pocket_type)
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

    fn container_data(&self, item: &ItemItem<'_>) -> Option<ContainerData> {
        let contents = self
            .pockets(item.entity, PocketType::Container)
            .collect::<Vec<_>>();
        let first_pocket_sealing = contents
            .first()
            .map(|first_subitems| first_subitems.pocket.sealing);
        first_pocket_sealing.map(|sealing| ContainerData {
            sealing,
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
        item: &ItemItem<'_>,
        container_data: Option<&mut ContainerData>,
    ) -> Vec<Fragment> {
        let magazine_wells = self.pockets(item.entity, PocketType::MagazineWell);

        let mut magazine_output = self
            .pockets(item.entity, PocketType::Magazine)
            .flat_map(|subitems| subitems.items)
            .map(|subitem| self.inline_item_fragments(&subitem))
            .collect::<Vec<_>>();

        let phrase = Phrase::from_fragments(prefix.into_iter().collect())
            .extend(item.fragments())
            .debug(format!("[{}]", item.common_info.id.fallback_name()))
            .extend(Self::battery_charge_fragments(item, &mut magazine_output))
            .extend(self.magazine_fragments(magazine_wells, &magazine_output))
            .extend(Self::item_tag_fragments(item, &container_data));

        if let Some(container_data) = container_data {
            if container_data.contents.is_empty() {
                phrase
            } else if self.is_single_hierarchy(&container_data.contents) {
                phrase.push(Fragment::good(">")).extend(
                    container_data
                        .contents
                        .drain(0..=0)
                        .flat_map(|subitem| self.inline_item_fragments(&subitem)),
                )
            } else {
                phrase.push(Fragment::good(format!(
                    "> {}+",
                    container_data.contents.len()
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
        magazine_output: &mut Vec<Vec<Fragment>>,
    ) -> impl Iterator<Item = Fragment> {
        item.magazine_info
            .filter(|magazine| {
                magazine
                    .ammo_type
                    .0
                    .contains(&UntypedInfoId::new("battery"))
            })
            .map(|magazine| {
                let mut charges = magazine_output.drain(..).flatten().peekable();
                if charges.peek().is_some() {
                    once(Fragment::soft("at"))
                        .chain(charges)
                        .chain(once(Fragment::soft(
                            magazine
                                .capacity
                                .map_or_else(String::new, |capacity| format!("/{capacity}")),
                        )))
                        .collect::<Vec<_>>()
                } else {
                    vec![
                        Fragment::soft("("),
                        Fragment::bad("drained"),
                        Fragment::soft(")"),
                    ]
                }
            })
            .into_iter()
            .flatten()
    }

    fn magazine_fragments<'a>(
        &self,
        magazine_wells: impl Iterator<Item = Subitems<'a>>,
        magazine_output: &[Vec<Fragment>],
    ) -> Vec<Fragment> {
        let magazine_framents = magazine_output.join(&Fragment::soft(", "));
        let well_fragments = magazine_wells
            .flat_map(|info| {
                if info.items.is_empty() {
                    vec![Fragment::soft("not loaded")]
                } else {
                    info.items
                        .iter()
                        .map(|subitem| self.inline_item_fragments(subitem))
                        .collect::<Vec<_>>()
                        .join(&Fragment::soft(", "))
                }
            })
            .collect::<Vec<_>>();

        if magazine_framents.is_empty() && well_fragments.is_empty() {
            Vec::new()
        } else {
            let both_used = !magazine_framents.is_empty() && !well_fragments.is_empty();
            let mut result = vec![Fragment::soft("with")];
            result.extend(magazine_framents);
            if both_used {
                result.push(Fragment::soft(", "));
            }
            result.extend(well_fragments);
            result
        }
    }

    fn item_tag_fragments(
        item: &ItemItem<'_>,
        container_data: &Option<&mut ContainerData<'_>>,
    ) -> Vec<Fragment> {
        let mut tags = Vec::new();
        if let Some(ref container_data) = *container_data {
            if container_data.contents.is_empty() {
                tags.push([Fragment::soft("empty")]);
            }
            tags.extend(container_data.sealing.suffix().map(|tag| [tag]));
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

    fn inline_item_fragments(&self, item: &ItemItem<'_>) -> Vec<Fragment> {
        let mut container_data = self.container_data(item);
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
                    .pockets(item.entity, PocketType::Container)
                    .flat_map(|pocket| pocket.items)
                    .collect::<Vec<_>>();
                self.is_single_hierarchy(&subitems)
            }
            _ => false,
        }
    }

    fn pockets(
        &self,
        item_entity: Entity,
        pocket_type: PocketType,
    ) -> impl Iterator<Item = Subitems> + use<'_> {
        self.pockets_in(item_entity)
            .filter(move |(_, pocket)| pocket_type == pocket.type_)
            .map(move |(pocket_entity, pocket)| Subitems {
                pocket,
                items: self.items_in(pocket_entity).collect::<Vec<_>>(),
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
