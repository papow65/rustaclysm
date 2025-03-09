use crate::{CommonItemInfo, LinkedLater, ObjectId, TerrainInfo};
use crate::{Flags, HashMap, ItemName};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct FurnitureInfo {
    pub name: ItemName,
    pub move_cost_mod: Option<MoveCostMod>,
    pub looks_like: Option<ObjectId>,
    pub flags: Flags,
    pub bash: Option<Bash>,

    /// Use [`Self.crafting_pseudo_item`] where possible
    #[serde(rename(deserialize = "crafting_pseudo_item"))]
    pub crafting_pseudo_item_id: Option<ObjectId>,
    #[serde(skip)]
    pub crafting_pseudo_item: Option<Arc<CommonItemInfo>>,

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
    #[serde(rename(deserialize = "ter_set"))]
    pub terrain: Option<LinkedLater<TerrainInfo>>,

    #[serde(rename(deserialize = "furn_set"))]
    pub furniture: Option<ObjectId>,
    pub items: Option<BashItems>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BashItems {
    Explicit(Vec<BashItem>),
    Collection(ObjectId),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BashItem {
    Single(ItemOccurrence),
    Group { group: ObjectId },
}

#[derive(Debug, Deserialize)]
pub struct ItemOccurrence {
    pub item: ObjectId,
    pub charges: Option<CountRange>,
    pub count: Option<CountRange>,
    pub prob: Option<Probability>,
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
