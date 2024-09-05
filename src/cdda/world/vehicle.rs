use crate::{cdda::CddaItem, gameplay::ObjectId};
use serde::Deserialize;

#[expect(unused, clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaVehicle {
    #[serde(rename = "type")]
    pub(crate) id: ObjectId,

    /// u8 would suffice, but i32 requires less casting
    pub(crate) posx: i32,
    /// u8 would suffice, but i32 requires less casting
    pub(crate) posy: i32,

    pub(crate) om_id: u8,
    #[serde(rename = "faceDir")]
    pub(crate) face_dir: u16,
    #[serde(rename = "moveDir")]
    pub(crate) move_dir: u16,
    pub(crate) turn_dir: u16,
    pub(crate) last_turn: u8,
    pub(crate) velocity: u8,
    pub(crate) avg_velocity: u8,
    pub(crate) falling: bool,
    pub(crate) in_water: bool,
    pub(crate) floating: bool,
    pub(crate) flying: bool,
    pub(crate) cruise_velocity: u16,
    pub(crate) vertical_velocity: u8,
    pub(crate) cruise_on: bool,
    pub(crate) engine_on: bool,
    pub(crate) tracking_on: bool,
    pub(crate) skidding: bool,
    pub(crate) of_turn_carry: f32,
    pub(crate) name: String,
    pub(crate) owner: String,
    pub(crate) old_owner: String,
    pub(crate) theft_time: Option<serde_json::Value>,
    pub(crate) parts: Vec<CddaVehiclePart>,
    pub(crate) tags: Vec<()>,
    pub(crate) fuel_remainder: serde_json::Value,
    pub(crate) fuel_used_last_turn: serde_json::Value,
    pub(crate) labels: Vec<()>,
    pub(crate) zones: Vec<()>,
    pub(crate) other_tow_point: (u8, u8, u8),
    pub(crate) is_locked: bool,
    pub(crate) is_alarm_on: bool,
    pub(crate) camera_on: bool,
    pub(crate) autopilot_on: bool,
    pub(crate) last_update_turn: u64,
    pub(crate) pivot: (i8, i8),
    pub(crate) is_on_ramp: bool,
    pub(crate) is_autodriving: bool,
    pub(crate) is_following: bool,
    pub(crate) is_patrolling: bool,
    pub(crate) autodrive_local_target: (u8, u8, u8),
    pub(crate) airworthy: bool,
    pub(crate) requested_z_change: u8,
    pub(crate) summon_time_limit: Option<()>,
    pub(crate) magic: bool,
    pub(crate) smart_controller: Option<()>,
    pub(crate) vehicle_noise: u8,
}

#[expect(unused)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaVehiclePart {
    pub(crate) id: ObjectId,
    pub(crate) variant: Option<String>,
    pub(crate) base: CddaItem,

    /// i8 would suffice, but i32 requires less casting
    pub(crate) mount_dx: i32,
    /// i8 would suffice, but i32 requires less casting
    pub(crate) mount_dy: i32,

    pub(crate) open: bool,
    pub(crate) direction: u8,
    pub(crate) blood: i16,
    pub(crate) enabled: bool,
    pub(crate) flags: u8,
    pub(crate) passenger_id: i8,
    pub(crate) crew_id: i8,
    pub(crate) items: Vec<CddaItem>,
    pub(crate) ammo_pref: String,
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
