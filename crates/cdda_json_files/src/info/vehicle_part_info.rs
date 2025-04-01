use crate::{CommonItemInfo, Ignored, InfoId, RequiredLinkedLater, UntypedInfoId};
use crate::{Flags, ItemName};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct VehiclePartInfo {
    pub id: InfoId<Self>,
    pub name: Option<ItemName>,
    pub item: RequiredLinkedLater<CommonItemInfo>,
    pub looks_like: Option<UntypedInfoId>,
    pub flags: Flags,

    pub backfire_freq: Option<u8>,
    pub backfire_threshold: Option<f32>,
    pub bonus: Option<u32>,
    pub breaks_into: Option<serde_json::Value>,
    pub broken_color: Option<Arc<str>>,
    pub broken_symbol: Option<Arc<str>>,
    pub categories: Option<Vec<serde_json::Value>>,
    pub color: Option<Arc<str>>,
    pub comfort: Option<u8>,
    pub contact_area: Option<u16>,
    pub damage_modifier: Option<u16>,
    pub damage_reduction: Option<serde_json::Value>,
    pub damaged_power_factor: Option<f32>,
    pub delete: Option<serde_json::Value>,
    pub description: Option<Arc<str>>,
    pub durability: Option<u16>,
    pub emissions: Option<Vec<serde_json::Value>>,
    pub energy_consumption: Option<Arc<str>>,
    pub epower: Option<i32>,
    pub exclusions: Option<Vec<serde_json::Value>>,
    pub exhaust: Option<Vec<serde_json::Value>>,
    pub extend: Option<serde_json::Value>,
    pub floor_bedding_warmth: Option<u16>,
    pub folded_volume: Option<Arc<str>>,
    pub folding_time: Option<Arc<str>>,
    pub fuel_options: Option<Vec<serde_json::Value>>,
    pub fuel_type: Option<Arc<str>>,
    pub location: Option<Arc<str>>,
    pub m2c: Option<u8>,
    pub muscle_power_factor: Option<u8>,
    pub noise_factor: Option<u8>,
    pub power: Option<i32>,
    pub proportional: Option<serde_json::Value>,
    pub pseudo_tools: Option<Vec<serde_json::Value>>,
    pub qualities: Option<Vec<serde_json::Value>>,
    pub requirements: Option<serde_json::Value>,
    pub rolling_resistance: Option<f32>,
    pub rotor_diameter: Option<u8>,
    pub size: Option<Arc<str>>,
    pub standard_symbols: Option<bool>,
    pub symbol: Option<Arc<str>>,
    pub symbols: Option<serde_json::Value>,
    pub transform_terrain: Option<serde_json::Value>,
    pub unfolding_time: Option<Arc<str>>,
    pub unfolding_tools: Option<Vec<serde_json::Value>>,
    pub wheel_type: Option<Arc<str>>,
    pub workbench: Option<serde_json::Value>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[cfg(test)]
mod item_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_train_motor.json");
        let result = serde_json::from_str::<VehiclePartInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
