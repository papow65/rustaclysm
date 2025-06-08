use crate::CddaPlayer;
use serde::Deserialize;
use serde_json::Value as JsonValue;

/// This represents a .sav file
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sav {
    pub achievements_tracker: JsonValue,

    pub active_monsters: JsonValue,
    pub auto_travel_mode: JsonValue,

    #[serde(rename = "bVMonsterLookFire")]
    pub b_v_monster_look_fire: JsonValue,
    pub calendar_start: u64,
    pub game_start: u64,

    pub turn: u64,

    pub driving_view_offset: JsonValue,
    pub global_vals: JsonValue,
    pub grscent: JsonValue,
    pub inactive_global_effect_on_condition_vector: JsonValue,
    pub initial_season: JsonValue,
    pub kill_tracker: JsonValue,

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

    pub mostseen: JsonValue,

    pub player: CddaPlayer,

    pub player_messages: JsonValue,
    pub queued_global_effect_on_conditions: JsonValue,
    pub run_mode: JsonValue,
    pub stats_tracker: JsonValue,
    pub turnssincelastmon: JsonValue,
    pub typescent: JsonValue,
    pub unique_npcs: Option<JsonValue>,
    pub view_offset_x: JsonValue,
    pub view_offset_y: JsonValue,
    pub view_offset_z: JsonValue,
}
