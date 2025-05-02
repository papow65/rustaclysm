use crate::{Amount, BodyContainers, LastSeen, Location, Pos, Shared};
use bevy::ecs::query::QueryData;
use bevy::platform::collections::{HashMap, HashSet, hash_map::Entry};
use bevy::prelude::{AnyOf, ChildOf, Children, Entity, Query};
use cdda_json_files::{
    CommonItemInfo, ExamineAction, FurnitureInfo, InfoId, Quality, SimpleExamineAction, TerrainInfo,
};
use std::{ops::RangeInclusive, sync::Arc};

const MAX_FIND_DISTANCE: i32 = 7;
const FIND_RANGE: RangeInclusive<i32> = (-MAX_FIND_DISTANCE)..=MAX_FIND_DISTANCE;

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub(super) struct Nearby {
    pub(super) entity: Entity,
    pub(super) amount: &'static Amount,
    pub(super) common_item_info: &'static Shared<CommonItemInfo>,
    pub(super) children: Option<&'static Children>,
}

pub(super) fn find_nearby<'a>(
    location: &'a Location,
    items: &'a Query<(Nearby, &LastSeen, Option<&ChildOf>)>,
    player_pos: Pos,
    body_containers: &'a BodyContainers,
) -> Vec<NearbyItem<'a>> {
    FIND_RANGE
        .flat_map(move |dz| {
            FIND_RANGE.flat_map(move |dx| {
                location
                    .all(player_pos.horizontal_offset(dx, dz))
                    .filter_map(|entity| items.get(*entity).ok())
                    .filter(|(.., last_seen, _)| **last_seen != LastSeen::Never)
            })
        })
        .chain(items.iter().filter(|(.., parent)| {
            parent.is_some_and(|child_of| {
                [body_containers.hands, body_containers.clothing].contains(&child_of.parent())
            })
        }))
        .map(|(nearby, ..)| nearby)
        .collect()
}

pub(super) fn find_nearby_pseudo(
    location: &Location,
    infrastructure: &Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
    player_pos: Pos,
) -> HashSet<Arc<CommonItemInfo>> {
    FIND_RANGE
        .flat_map(move |dz| {
            FIND_RANGE.flat_map(move |dx| {
                location
                    .all(player_pos.horizontal_offset(dx, dz))
                    .filter_map(|entity| infrastructure.get(*entity).ok())
                    .filter(|(.., last_seen)| **last_seen != LastSeen::Never)
            })
        })
        .filter_map(|((furniture_info, _), ..)| {
            furniture_info.and_then(|f| f.crafting_pseudo_item.get())
        })
        .collect()
}

pub(super) fn find_sources(
    location: &Location,
    infrastructure: &Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
    player_pos: Pos,
) -> HashSet<InfoId<CommonItemInfo>> {
    FIND_RANGE
        .flat_map(move |dz| {
            FIND_RANGE.flat_map(move |dx| {
                location
                    .all(player_pos.horizontal_offset(dx, dz))
                    .filter_map(|entity| infrastructure.get(*entity).ok())
                    .filter(|(.., last_seen)| **last_seen != LastSeen::Never)
            })
        })
        .filter_map(|(nearby, ..)| {
            nearby
                .0
                .and_then(|furniture_info| furniture_info.examine_action.0.as_ref())
                .or_else(|| {
                    nearby
                        .1
                        .and_then(|terrain_info| terrain_info.examine_action.0.as_ref())
                })
        })
        .filter_map(|examine_action| {
            matches!(
                examine_action,
                ExamineAction::Simple(SimpleExamineAction::WaterSource)
            )
            .then_some(InfoId::new("water"))
        })
        .collect()
}

pub(super) fn nearby_qualities(
    nearby_items: &[NearbyItem],
    pseudo_items: &HashSet<Arc<CommonItemInfo>>,
) -> HashMap<Arc<Quality>, i8> {
    nearby_items
        .iter()
        .map(|nearby| nearby.common_item_info.as_ref().clone())
        .chain(pseudo_items.iter().cloned())
        .flat_map(|item| {
            item.qualities
                .iter()
                .filter_map(|item_quality| item_quality.as_tuple())
                .collect::<Vec<_>>()
        })
        .fold(
            HashMap::default(),
            |mut map: HashMap<Arc<Quality>, i8>, (quality, amount)| {
                match map.entry(quality) {
                    Entry::Occupied(occ) => {
                        *occ.into_mut() = (*occ.get()).max(amount);
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(amount);
                    }
                }
                map
            },
        )
}
