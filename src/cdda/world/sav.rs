use crate::prelude::{CddaPlayer, PathFor};
use bevy::ecs::system::Resource;
use serde::Deserialize;
use std::fs::read_to_string;

pub(crate) type SavPath = PathFor<Sav>;

/// This represents a .sav file
#[derive(Debug, Deserialize, Resource)]
#[serde(deny_unknown_fields)]
pub(crate) struct Sav {
    #[allow(unused)]
    pub(crate) achievements_tracker: serde_json::Value,

    #[allow(unused)]
    pub(crate) active_monsters: serde_json::Value,

    #[allow(unused)]
    pub(crate) auto_travel_mode: serde_json::Value,

    #[allow(unused)]
    #[serde(rename(deserialize = "bVMonsterLookFire"))]
    pub(crate) b_v_monster_look_fire: serde_json::Value,

    #[allow(unused)]
    pub(crate) calendar_start: u64,

    #[allow(unused)]
    pub(crate) game_start: u64,

    pub(crate) turn: u64,

    #[allow(unused)]
    pub(crate) driving_view_offset: serde_json::Value,

    #[allow(unused)]
    pub(crate) global_vals: serde_json::Value,

    #[allow(unused)]
    pub(crate) grscent: serde_json::Value,

    #[allow(unused)]
    pub(crate) inactive_global_effect_on_condition_vector: serde_json::Value,

    #[allow(unused)]
    pub(crate) initial_season: serde_json::Value,

    #[allow(unused)]
    pub(crate) kill_tracker: serde_json::Value,

    /** Overmap x */
    pub(crate) om_x: i16,
    /** Overmap y -> z */
    pub(crate) om_y: i16,

    /** Subzone coordinate x, 0 <= levx < 360 */
    pub(crate) levx: u16,
    /** Subzone coordinate y -> z, 0 <= levy < 360 */
    pub(crate) levy: u16,
    /** Level */
    pub(crate) levz: i8,

    #[allow(unused)]
    pub(crate) mostseen: serde_json::Value,

    #[allow(unused)]
    pub(crate) player: CddaPlayer,

    #[allow(unused)]
    pub(crate) player_messages: serde_json::Value,

    #[allow(unused)]
    pub(crate) queued_global_effect_on_conditions: serde_json::Value,

    #[allow(unused)]
    pub(crate) run_mode: serde_json::Value,

    #[allow(unused)]
    pub(crate) stats_tracker: serde_json::Value,

    #[allow(unused)]
    pub(crate) turnssincelastmon: serde_json::Value,

    #[allow(unused)]
    pub(crate) typescent: serde_json::Value,

    #[allow(unused)]
    pub(crate) unique_npcs: Option<serde_json::Value>,

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
            .map(|s| String::from(s.split_at(s.find('\n').unwrap()).1))
            .map(|s| serde_json::from_str::<Self>(s.as_str()))
            .expect(".sav file could not be read")
    }
}
