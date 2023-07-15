use crate::prelude::{Flags, ItemName, ObjectId};
use bevy::utils::HashMap;
use rand::{thread_rng, Rng};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct FurnitureInfo {
    pub(crate) name: ItemName,
    pub(crate) move_cost_mod: Option<MoveCostMod>,
    pub(crate) looks_like: Option<ObjectId>,

    #[serde(default)]
    pub(crate) flags: Flags,

    pub(crate) bash: Option<Bash>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub(crate) enum MoveCostMod {
    Slower(MoveCostIncrease),
    Impossible(i8), // -1
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, PartialOrd)]
pub(crate) struct MoveCostIncrease(pub(crate) u8);

#[derive(Debug, Deserialize)]
pub(crate) struct Bash {
    pub(crate) ter_set: Option<ObjectId>,
    pub(crate) furn_set: Option<ObjectId>,
    pub(crate) items: Option<BashItems>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum BashItems {
    Explicit(Vec<BashItem>),
    Collection(ObjectId),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum BashItem {
    Single(ItemOccurrence),
    Group { group: ObjectId },
}

#[derive(Debug, Deserialize)]
pub(crate) struct ItemOccurrence {
    pub(crate) item: ObjectId,
    pub(crate) charges: Option<CountRange>,
    pub(crate) count: Option<CountRange>,
    pub(crate) prob: Option<Probability>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum CountRange {
    Fixed(u32),
    FromTo(u32, u32),
}

impl CountRange {
    pub(crate) fn random(&self) -> u32 {
        match self {
            Self::Fixed(fixed) => *fixed,
            Self::FromTo(from, to) => thread_rng().gen_range(*from..=*to),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Probability(u8);

impl Probability {
    pub(crate) fn random(&self) -> bool {
        thread_rng().gen_range(0..=100) <= self.0
    }
}

#[cfg(test)]
mod container_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("bash.json");
        let result = serde_json::from_str::<Bash>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
