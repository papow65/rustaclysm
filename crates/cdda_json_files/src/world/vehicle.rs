use crate::{CddaItem, ObjectId};
use serde::Deserialize;
use std::sync::Arc;

#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaVehicle {
    #[serde(rename = "type")]
    pub id: ObjectId,

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
    pub last_turn: u8,
    pub velocity: u8,
    pub avg_velocity: u8,
    pub falling: bool,
    pub in_water: bool,
    pub floating: bool,
    pub flying: bool,
    pub cruise_velocity: u16,
    pub vertical_velocity: u8,
    pub cruise_on: bool,
    pub engine_on: bool,
    pub tracking_on: bool,
    pub skidding: bool,
    pub of_turn_carry: f32,
    pub name: Arc<str>,
    pub owner: Arc<str>,
    pub old_owner: Arc<str>,
    pub theft_time: Option<serde_json::Value>,
    pub parts: Vec<CddaVehiclePart>,
    pub tags: Vec<()>,
    pub fuel_remainder: serde_json::Value,
    pub fuel_used_last_turn: serde_json::Value,
    pub labels: Vec<()>,
    pub zones: Vec<()>,
    pub other_tow_point: (u8, u8, u8),
    pub is_locked: bool,
    pub is_alarm_on: bool,
    pub camera_on: bool,
    pub autopilot_on: bool,
    pub last_update_turn: u64,
    pub pivot: (i8, i8),
    pub is_on_ramp: bool,
    pub is_autodriving: bool,
    pub is_following: bool,
    pub is_patrolling: bool,
    pub autodrive_local_target: (u8, u8, u8),
    pub airworthy: bool,
    pub requested_z_change: u8,
    pub summon_time_limit: Option<()>,
    pub magic: bool,
    pub smart_controller: Option<()>,
    pub vehicle_noise: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaVehiclePart {
    pub id: ObjectId,
    pub variant: Option<Arc<str>>,
    pub base: CddaItem,

    /// i8 would suffice, but i32 requires less casting
    pub mount_dx: i32,
    /// i8 would suffice, but i32 requires less casting
    pub mount_dy: i32,

    pub open: bool,
    pub direction: u8,
    pub blood: i16,
    pub enabled: bool,
    pub flags: u8,
    pub passenger_id: i8,
    pub crew_id: i8,
    pub items: Vec<CddaItem>,
    pub ammo_pref: Arc<str>,
}

#[cfg(test)]
mod vehicle_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_vehicle.json");
        let result = serde_json::from_str::<CddaVehicle>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
