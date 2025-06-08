use crate::{CommonItemInfo, Ignored, InfoId, RequiredLinkedLater, UntypedInfoId};
use crate::{Flags, ItemName};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct VehiclePartInfo {
    pub id: InfoId<Self>,
    pub name: Option<ItemName>,
    pub item: RequiredLinkedLater<CommonItemInfo>,
    pub categories: Vec<JsonValue>,
    pub durability: u16,

    pub looks_like: Option<UntypedInfoId>,
    pub flags: Flags,

    pub backfire_freq: Option<u8>,
    pub backfire_threshold: Option<f32>,
    pub bonus: Option<u32>,
    pub breaks_into: Option<JsonValue>,
    pub broken_color: Option<Arc<str>>,
    pub broken_symbol: Option<Arc<str>>,
    pub color: Option<Arc<str>>,
    pub comfort: Option<u8>,
    pub contact_area: Option<u16>,
    pub damage_modifier: Option<u16>,
    pub damage_reduction: Option<JsonValue>,
    pub damaged_power_factor: Option<f32>,
    pub delete: Option<JsonValue>,
    pub description: Option<Arc<str>>,
    pub emissions: Option<Vec<JsonValue>>,
    pub energy_consumption: Option<Arc<str>>,
    pub epower: Option<i32>,
    pub exclusions: Option<Vec<JsonValue>>,
    pub exhaust: Option<Vec<JsonValue>>,
    pub extend: Option<JsonValue>,
    pub floor_bedding_warmth: Option<u16>,
    pub folded_volume: Option<Arc<str>>,
    pub folding_time: Option<Arc<str>>,
    pub fuel_options: Option<Vec<JsonValue>>,
    pub fuel_type: Option<Arc<str>>,
    pub location: Option<Arc<str>>,
    pub m2c: Option<u8>,
    pub muscle_power_factor: Option<u8>,
    pub noise_factor: Option<u8>,
    pub power: Option<i32>,
    pub proportional: Option<JsonValue>,
    pub pseudo_tools: Option<Vec<JsonValue>>,
    pub qualities: Option<Vec<JsonValue>>,
    pub requirements: Option<JsonValue>,
    pub rolling_resistance: Option<f32>,
    pub rotor_diameter: Option<u8>,
    pub size: Option<Arc<str>>,
    pub standard_symbols: Option<bool>,
    pub symbol: Option<Arc<str>>,
    pub symbols: Option<JsonValue>,
    pub transform_terrain: Option<JsonValue>,
    pub unfolding_time: Option<Arc<str>>,
    pub unfolding_tools: Option<Vec<JsonValue>>,
    pub wheel_type: Option<Arc<str>>,
    pub workbench: Option<JsonValue>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[cfg(test)]
mod item_tests {
    use super::*;
    use serde_json::from_str as from_json_str;

    #[test]
    fn it_works() {
        let json = include_str!("test_train_motor.json");
        let result = from_json_str::<VehiclePartInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
