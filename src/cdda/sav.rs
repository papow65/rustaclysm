use crate::prelude::PathFor;
use serde::Deserialize;
use std::fs::read_to_string;

pub(crate) type SavPath = PathFor<Sav>;

/// This represents a sav-file
#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Sav {
    achievements_tracker: serde_json::Value,
    active_monsters: serde_json::Value,
    auto_travel_mode: serde_json::Value,

    #[serde(rename(deserialize = "bVMonsterLookFire"))]
    b_v_monster_look_fire: serde_json::Value,

    calendar_start: serde_json::Value,
    driving_view_offset: serde_json::Value,
    game_start: serde_json::Value,
    global_vals: serde_json::Value,
    grscent: serde_json::Value,
    inactive_global_effect_on_condition_vector: serde_json::Value,
    initial_season: serde_json::Value,
    kill_tracker: serde_json::Value,
    levx: serde_json::Value,
    levy: serde_json::Value,
    levz: serde_json::Value,
    mostseen: serde_json::Value,
    om_x: i16,
    om_y: i16,
    player: serde_json::Value,
    player_messages: serde_json::Value,
    queued_global_effect_on_conditions: serde_json::Value,
    run_mode: serde_json::Value,
    stats_tracker: serde_json::Value,
    turn: serde_json::Value,
    turnssincelastmon: serde_json::Value,
    typescent: serde_json::Value,
    view_offset_x: serde_json::Value,
    view_offset_y: serde_json::Value,
    view_offset_z: serde_json::Value,
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
            .expect("sav file could not be read")
    }
}
