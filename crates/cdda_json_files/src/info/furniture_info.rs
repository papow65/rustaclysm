use crate::{
    CommonItemInfo, Flags, HashMap, InfoId, ItemGroup, ItemName, OptionalLinkedLater,
    RequiredLinkedLater, SpawnItem, TerrainInfo, UntypedInfoId,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct FurnitureInfo {
    pub id: InfoId<Self>,
    pub name: ItemName,
    pub move_cost_mod: Option<MoveCostMod>,
    pub looks_like: Option<UntypedInfoId>,
    pub flags: Flags,
    pub bash: Option<Bash>,
    pub crafting_pseudo_item: OptionalLinkedLater<CommonItemInfo>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum MoveCostMod {
    Slower(MoveCostIncrease),
    Impossible(i8), // -1
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, PartialOrd)]
pub struct MoveCostIncrease(pub u8);

#[derive(Debug, Deserialize)]
pub struct Bash {
    #[serde(rename = "ter_set")]
    pub terrain: OptionalLinkedLater<TerrainInfo>,

    #[serde(rename = "furn_set")]
    pub furniture: OptionalLinkedLater<FurnitureInfo>,

    pub items: Option<BashItems>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BashItems {
    Explicit(Vec<BashItem>),
    Collection(RequiredLinkedLater<ItemGroup>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BashItem {
    Single(ItemOccurrence),
    Group {
        group: RequiredLinkedLater<ItemGroup>,
    },
}

impl BashItem {
    pub fn items(&self) -> Vec<SpawnItem> {
        match self {
            Self::Single(occurrence) => occurrence.items().collect(),
            Self::Group { group } => group
                .get_option("item group from bashed item")
                .into_iter()
                .flat_map(|item_group| item_group.items().collect::<Vec<_>>())
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ItemOccurrence {
    pub item: RequiredLinkedLater<CommonItemInfo>,
    pub charges: Option<CountRange>,
    pub count: Option<CountRange>,
    pub prob: Option<Probability>,
}

impl ItemOccurrence {
    fn items(&self) -> impl Iterator<Item = SpawnItem> {
        self.prob
            .as_ref()
            .map(|prob| prob.random())
            .unwrap_or(true)
            .then_some(self.item.get_option("item occurrence from bashed item"))
            .flatten()
            .into_iter()
            .map(|item| SpawnItem {
                item_info: item,
                amount: self.count.as_ref().map(|count| count.random()).unwrap_or(1),
                charges: self.charges.as_ref().map(|charges| charges.random()),
            })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CountRange {
    Fixed(u32),
    FromTo(u32, u32),
}

impl CountRange {
    #[must_use]
    pub fn random(&self) -> u32 {
        match self {
            Self::Fixed(fixed) => *fixed,
            Self::FromTo(from, to) => fastrand::u32(*from..=*to),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Probability(u8);

impl Probability {
    pub fn random(&self) -> bool {
        fastrand::u8(0..100) < self.0
    }
}

#[cfg(test)]
mod furniture_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_bash.json");
        let result = serde_json::from_str::<Bash>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
