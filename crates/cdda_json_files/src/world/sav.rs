use crate::CddaPlayer;
use serde::Deserialize;

/// This represents a .sav file
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sav {
    pub achievements_tracker: serde_json::Value,

    pub active_monsters: serde_json::Value,
    pub auto_travel_mode: serde_json::Value,

    #[serde(rename(deserialize = "bVMonsterLookFire"))]
    pub b_v_monster_look_fire: serde_json::Value,
    pub calendar_start: u64,
    pub game_start: u64,

    pub turn: u64,

    pub driving_view_offset: serde_json::Value,
    pub global_vals: serde_json::Value,
    pub grscent: serde_json::Value,
    pub inactive_global_effect_on_condition_vector: serde_json::Value,
    pub initial_season: serde_json::Value,
    pub kill_tracker: serde_json::Value,

    /// Overmap x
    pub om_x: i16,
    /// Overmap y -> z
    pub om_y: i16,

    /// Subzone coordinate x, 0 <= levx < 360
    pub levx: u16,
    /// Subzone coordinate y -> z, 0 <= levy < 360
    pub levy: u16,
    /// Level
    pub levz: i8,

    pub mostseen: serde_json::Value,

    pub player: CddaPlayer,

    pub player_messages: serde_json::Value,
    pub queued_global_effect_on_conditions: serde_json::Value,
    pub run_mode: serde_json::Value,
    pub stats_tracker: serde_json::Value,
    pub turnssincelastmon: serde_json::Value,
    pub typescent: serde_json::Value,
    pub unique_npcs: Option<serde_json::Value>,
    pub view_offset_x: serde_json::Value,
    pub view_offset_y: serde_json::Value,
    pub view_offset_z: serde_json::Value,
}
