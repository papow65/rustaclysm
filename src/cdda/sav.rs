use crate::prelude::PathFor;
use serde::Deserialize;
use std::fs::read_to_string;

pub(crate) type SavPath = PathFor<Sav>;

/// This represents a .sav file
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Sav {
    #[allow(unused)]
    achievements_tracker: serde_json::Value,

    #[allow(unused)]
    active_monsters: serde_json::Value,

    #[allow(unused)]
    auto_travel_mode: serde_json::Value,

    #[allow(unused)]
    #[serde(rename(deserialize = "bVMonsterLookFire"))]
    b_v_monster_look_fire: serde_json::Value,

    #[allow(unused)]
    pub(crate) calendar_start: u64,

    #[allow(unused)]
    pub(crate) game_start: u64,

    pub(crate) turn: u64,

    #[allow(unused)]
    driving_view_offset: serde_json::Value,

    #[allow(unused)]
    global_vals: serde_json::Value,

    #[allow(unused)]
    grscent: serde_json::Value,

    #[allow(unused)]
    inactive_global_effect_on_condition_vector: serde_json::Value,

    #[allow(unused)]
    initial_season: serde_json::Value,

    #[allow(unused)]
    kill_tracker: serde_json::Value,

    pub(crate) om_x: i16,
    pub(crate) om_y: i16,

    pub(crate) levx: i16,
    pub(crate) levy: i16,
    pub(crate) levz: i8,

    #[allow(unused)]
    mostseen: serde_json::Value,

    #[allow(unused)]
    player: serde_json::Value,

    #[allow(unused)]
    player_messages: serde_json::Value,

    #[allow(unused)]
    queued_global_effect_on_conditions: serde_json::Value,

    #[allow(unused)]
    run_mode: serde_json::Value,

    #[allow(unused)]
    stats_tracker: serde_json::Value,

    #[allow(unused)]
    turnssincelastmon: serde_json::Value,

    #[allow(unused)]
    typescent: serde_json::Value,

    #[allow(unused)]
    unique_npcs: Option<serde_json::Value>,

    #[allow(unused)]
    pub(crate) view_offset_x: serde_json::Value,

    #[allow(unused)]
    pub(crate) view_offset_y: serde_json::Value,

    #[allow(unused)]
    pub(crate) view_offset_z: serde_json::Value,
}

impl TryFrom<&SavPath> for Sav {
    type Error = serde_json::Error;
    fn try_from(sav_path: &SavPath) -> Result<Self, Self::Error> {
        read_to_string(&sav_path.0)
            .ok()
            .map(|s| {
                println!("Loading {}...", sav_path.0.display());
                s
            })
            .map(|s| s.split_at(s.find('\n').unwrap()).1.to_string())
            .map(|s| serde_json::from_str::<Self>(s.as_str()))
            .expect(".sav file could not be read")
    }
}
