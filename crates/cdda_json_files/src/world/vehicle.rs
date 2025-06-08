use crate::{CddaItem, Repetition, RequiredLinkedLater, UntypedInfoId, VehiclePartInfo};
use bevy_platform::collections::HashMap;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::sync::Arc;

#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaVehicle {
    #[serde(rename = "type")]
    pub id: UntypedInfoId,

    /// u8 would suffice, but i32 requires less casting
    pub posx: i32,
    /// u8 would suffice, but i32 requires less casting
    pub posy: i32,

    pub om_id: u8,
    #[serde(rename = "faceDir")]
    pub face_dir: u16,
    #[serde(rename = "moveDir")]
    pub move_dir: u16,
    pub turn_dir: u16,
    pub last_turn: Option<u8>,
    pub velocity: u8,
    pub avg_velocity: Option<u8>,
    pub falling: bool,

    #[serde(default)]
    pub in_water: bool,
    #[serde(default)]
    pub floating: bool,
    #[serde(default)]
    pub flying: bool,

    /// Usually positive, sometimes -400
    pub cruise_velocity: i16,

    pub vertical_velocity: u8,
    pub cruise_on: Option<bool>,
    pub engine_on: bool,
    pub tracking_on: bool,
    pub skidding: bool,
    pub of_turn_carry: f32,
    pub name: Arc<str>,
    pub owner: Arc<str>,
    pub old_owner: Arc<str>,
    pub theft_time: Option<JsonValue>,
    pub parts: Vec<CddaVehiclePart>,

    /// May contain "APPLICANCE"
    pub tags: Vec<String>,

    pub fuel_remainder: Option<JsonValue>,
    pub fuel_used_last_turn: Option<JsonValue>,
    pub labels: Vec<JsonValue>,

    // TODO for example:
    // {"point": [1,-1],"zone": {"name": "Loot: P.Food","type": "LOOT_PFOOD","faction": "your_followers",
    // "invert": false,"enabled": false,"is_vehicle": true,"is_personal":false,"cached_shift":[0,0,0],
    // "start":[-7803,2390,0],"end":[-7803,2390,0]}}
    pub zones: Vec<HashMap<String, JsonValue>>,

    pub other_tow_point: (u8, u8, u8),
    pub is_locked: bool,
    pub is_alarm_on: bool,
    pub camera_on: bool,

    #[serde(default)]
    pub autopilot_on: bool,

    pub last_update_turn: u64,
    pub pivot: (i8, i8),

    #[serde(default)]
    pub is_on_ramp: bool,
    #[serde(default)]
    pub is_autodriving: bool,

    pub is_following: bool,
    pub is_patrolling: bool,
    pub autodrive_local_target: (u8, u8, u8),
    pub airworthy: bool,

    #[serde(default)]
    pub requested_z_change: u8,

    pub summon_time_limit: Option<()>,
    pub magic: bool,
    pub smart_controller: Option<()>,

    #[serde(default)]
    pub vehicle_noise: Option<u8>,

    pub in_deep_water: Option<JsonValue>,
    pub precollision_on: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaVehiclePart {
    #[serde(rename = "id")]
    pub info: RequiredLinkedLater<VehiclePartInfo>,

    pub variant: Option<Arc<str>>,
    pub base: Arc<CddaItem>,

    /// i8 would suffice, but i32 requires less casting
    pub mount_dx: i32,
    /// i8 would suffice, but i32 requires less casting
    pub mount_dy: i32,

    pub open: bool,

    /// In degrees
    pub direction: u16,

    pub blood: i16,
    pub enabled: bool,
    pub flags: u8,
    #[serde(default)]
    pub carry: Vec<String>,
    pub passenger_id: i8,
    pub crew_id: i8,
    pub items: Vec<Repetition<CddaItem>>,
    pub target_first_x: Option<i16>,
    pub target_first_y: Option<i16>,
    pub target_first_z: Option<i16>,
    pub target_second_x: Option<i16>,
    pub target_second_y: Option<i16>,
    pub target_second_z: Option<i16>,
    pub ammo_pref: Arc<str>,

    pub tools: Option<JsonValue>,
    pub salvageable: Option<JsonValue>,
    pub locked: Option<JsonValue>,
    pub last_disconnected: Option<JsonValue>,
}

#[cfg(test)]
mod vehicle_tests {
    use super::*;
    use serde_json::from_str as from_json_str;

    #[test]
    fn it_works() {
        let json = include_str!("test_vehicle.json");
        let result = from_json_str::<CddaVehicle>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
