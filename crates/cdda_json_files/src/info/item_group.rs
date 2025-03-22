use crate::{CommonItemInfo, InfoId, RequiredLinkedLater};
use either::Either;
use fastrand::choose_multiple;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ItemGroup {
    pub id: InfoId<Self>,

    #[serde(flatten)]
    pub details: Option<ItemGroupDetails>,
}

impl ItemGroup {
    pub fn items(&self) -> impl Iterator<Item = SpawnItem> + use<'_> {
        self.details.iter().flat_map(ItemGroupDetails::items)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "subtype")]
pub enum ItemGroupDetails {
    #[serde(rename = "collection")]
    Collection { entries: Vec<ItemCollectionEntry> },

    #[serde(untagged)]
    #[serde(rename = "distribution")]
    Distribution {
        #[serde(alias = "items")] // Allow both 'items' and 'entries'
        entries: Vec<Probability>,
    },
}

impl ItemGroupDetails {
    pub fn items(&self) -> impl Iterator<Item = SpawnItem> + use<'_> {
        match self {
            Self::Collection { entries } => {
                let left = entries.iter().flat_map(ItemCollectionEntry::items);
                Either::Left(left)
            }
            Self::Distribution { entries } => {
                let right = entries.iter().flat_map(Probability::items);
                Either::Right(right)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ItemCollectionEntry {
    #[serde(flatten)]
    pub result: ItemOrGroup,

    #[serde(flatten)]
    pub count: Option<ItemCollectionEntryCount>,
}

impl ItemCollectionEntry {
    pub fn items(&self) -> impl Iterator<Item = SpawnItem> {
        self.result.items().into_iter().map(|mut spawn_item| {
            if let Some(count) = &self.count {
                spawn_item.amount *= count.random_amount();
                spawn_item.charges = count.random_charges();
            }
            spawn_item
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ItemCollectionEntryCount {
    Simple {
        count: u32,
    },
    Range {
        #[serde(rename = "count-min")]
        count_min: u32,

        #[serde(rename = "count-max")]
        count_max: u32,
    },
    Charged {
        #[serde(rename = "charges-min")]
        charges_min: u32,

        #[serde(rename = "charges-max")]
        charges_max: u32,
    },
}

impl ItemCollectionEntryCount {
    fn random_amount(&self) -> u32 {
        match self {
            Self::Simple { count } => *count,
            Self::Range {
                count_min,
                count_max,
            } => fastrand::u32(count_min..=count_max),
            Self::Charged { .. } => 1,
        }
    }

    fn random_charges(&self) -> Option<u32> {
        if let Self::Charged {
            charges_min,
            charges_max,
        } = *self
        {
            Some(fastrand::u32(charges_min..=charges_max))
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ItemOrGroup {
    Item {
        item: RequiredLinkedLater<CommonItemInfo>,
        variant: Option<Arc<str>>,
    },
    Group {
        group: RequiredLinkedLater<ItemGroup>,
    },
}

impl ItemOrGroup {
    pub fn items(&self) -> Vec<SpawnItem> {
        match self {
            Self::Item { item, .. } => item
                .get_option("item from item group")
                .into_iter()
                .map(|item_info| SpawnItem {
                    item_info,
                    amount: 1,
                    charges: None,
                })
                .collect(),
            Self::Group { group } => group
                .get_option("nested group of item group")
                .into_iter()
                .flat_map(|group| group.items().collect::<Vec<_>>())
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Probability {
    Structured {
        #[serde(flatten)]
        result: ItemOrGroup,
        prob: ProbabilityPercent,
        event: Option<Arc<str>>,
    },
    Array(RequiredLinkedLater<CommonItemInfo>, ProbabilityPercent),
}

impl Probability {
    pub fn items(&self) -> Vec<SpawnItem> {
        match self {
            Self::Structured { result, prob, .. } => result
                .items()
                .into_iter()
                .map(|mut spawn_intem| {
                    spawn_intem.amount = prob.random(spawn_intem.amount);
                    spawn_intem
                })
                .filter(|spawn_item| 0 < spawn_item.amount)
                .collect(),
            Self::Array(item, prob) => item
                .get_option("probable item from item group")
                .into_iter()
                .map(|item_info| SpawnItem {
                    item_info,
                    amount: prob.random(1),
                    charges: None,
                })
                .filter(|spawn_item| 0 < spawn_item.amount)
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProbabilityPercent(u8);

impl ProbabilityPercent {
    fn random(&self, tries: u32) -> u32 {
        let dice = choose_multiple(0..100_u8, tries as usize);
        dice.into_iter().filter(|roll| *roll <= self.0).count() as u32
    }
}

/// Calculated list of items to spawn
pub struct SpawnItem {
    pub item_info: Arc<CommonItemInfo>,
    pub amount: u32,
    pub charges: Option<u32>,
}

#[cfg(test)]
mod quality_tests {
    use super::*;

    #[test]
    fn lizard_sample_huge_works() {
        let json = include_str!("test_lizard_sample_huge.json");
        let result = serde_json::from_str::<ItemGroup>(json);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn molebot_works() {
        let json = include_str!("test_molebot.json");
        let result = serde_json::from_str::<ItemGroup>(json);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn swimmer_wetsuit_works() {
        let json = include_str!("test_swimmer_wetsuit.json");
        let result = serde_json::from_str::<ItemGroup>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
