use crate::{
    CommonItemInfo, ExamineActionOption, Flags, Ignored, InfoId, ItemGroup, ItemName,
    OptionalLinkedLater, RequiredLinkedLater, SpawnItem, TerrainInfo, UntypedInfoId,
};
use bevy_platform_support::collections::HashMap;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct FurnitureInfo {
    pub id: InfoId<Self>,
    pub name: ItemName,
    pub move_cost_mod: MoveCostMod,
    pub description: Arc<str>,
    pub required_str: i8,

    pub looks_like: Option<UntypedInfoId>,
    pub flags: Flags,
    pub bash: Option<Bash>,
    pub crafting_pseudo_item: OptionalLinkedLater<CommonItemInfo>,
    pub bgcolor: Option<serde_json::Value>,
    pub bonus_fire_warmth_feet: Option<u16>,
    pub close: Option<Arc<str>>,
    pub color: Option<serde_json::Value>,
    pub comfort: Option<u8>,
    pub connect_groups: Option<Arc<str>>,
    pub connects_to: Option<Arc<str>>,
    pub coverage: Option<u8>,
    pub deconstruct: Option<serde_json::Value>,
    pub deployed_item: Option<Arc<str>>,
    pub emissions: Option<Vec<serde_json::Value>>,
    pub examine_action: ExamineActionOption,
    pub floor_bedding_warmth: Option<i16>,
    pub hacksaw: Option<serde_json::Value>,
    pub harvest_by_season: Option<Vec<serde_json::Value>>,
    pub keg_capacity: Option<u16>,
    pub light_emitted: Option<u8>,
    pub lockpick_message: Option<Arc<str>>,
    pub lockpick_result: Option<Arc<str>>,
    pub max_volume: Option<Arc<str>>,
    pub open: Option<Arc<str>>,
    pub oxytorch: Option<serde_json::Value>,
    pub plant_data: Option<serde_json::Value>,
    pub prying: Option<serde_json::Value>,
    pub rotates_to: Option<Arc<str>>,
    pub shoot: Option<serde_json::Value>,
    pub surgery_skill_multiplier: Option<f32>,
    pub symbol: Option<Arc<str>>,
    pub workbench: Option<serde_json::Value>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
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
            .is_none_or(Probability::random)
            .then_some(self.item.get_option("item occurrence from bashed item"))
            .flatten()
            .into_iter()
            .map(|item| SpawnItem {
                item_info: item,
                amount: self.count.as_ref().map_or(1, CountRange::random),
                charges: self.charges.as_ref().map(CountRange::random),
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
