use crate::ObjectId;
use crate::{CddaAmount, FlatVec, RepetitionBlock};
use bevy::{asset::Asset, reflect::TypePath, utils::HashMap};
use serde::Deserialize;

/// Corresponds to an 'overmap' in CDDA. It defines the layout of 180x180 `Zone`s.
#[derive(Debug, Deserialize, Asset, TypePath)]
#[serde(deny_unknown_fields)]
pub struct Overmap {
    pub layers: [OvermapLevel; Self::LEVEL_AMOUNT],
    pub region_id: serde_json::Value,
    pub monster_groups: serde_json::Value,
    pub cities: serde_json::Value,
    pub connections_out: serde_json::Value,
    pub radios: serde_json::Value,
    pub monster_map: FlatVec<(SubzoneOffset, Monster), 2>,
    pub tracked_vehicles: serde_json::Value,
    pub scent_traces: serde_json::Value,
    pub npcs: serde_json::Value,
    pub camps: serde_json::Value,
    pub overmap_special_placements: serde_json::Value,
    pub mapgen_arg_storage: Option<serde_json::Value>,
    pub mapgen_arg_index: Option<serde_json::Value>,
    pub joins_used: Option<serde_json::Value>,
    pub predecessors: Option<serde_json::Value>,
}

impl Overmap {
    pub const LEVEL_AMOUNT: usize = 21;
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OvermapLevel(pub RepetitionBlock<ObjectId>);

impl OvermapLevel {
    pub fn all(id: ObjectId) -> Self {
        Self(RepetitionBlock::new(CddaAmount {
            obj: id,
            amount: 180 * 180,
        }))
    }
}

/// Offset of the subzone from the overmap
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SubzoneOffset(pub u16, pub u16, pub i8);

#[expect(unused)]
#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Monster {
    location: (i32, i32, i8),
    moves: i16,
    pain: u32,
    effects: HashMap<String, serde_json::Value>,
    damage_over_time_map: Vec<serde_json::Value>,
    values: HashMap<String, serde_json::Value>,
    blocks_left: u8,
    dodges_left: u8,
    num_blocks_bonus: u8,
    num_dodges_bonus: u8,
    armor_bash_bonus: u8,
    armor_cut_bonus: u8,
    armor_bullet_bonus: u8,
    speed: u16,
    speed_bonus: i16,
    dodge_bonus: f32,
    block_bonus: u8,
    hit_bonus: f32,
    bash_bonus: u8,
    cut_bonus: u8,
    bash_mult: f32,
    cut_mult: f32,
    melee_quiet: bool,
    throw_resist: u8,
    archery_aim_counter: u8,
    last_updated: u32,
    body: HashMap<String, serde_json::Value>,
    pub typeid: ObjectId,
    unique_name: String,
    nickname: String,
    goal: Option<serde_json::Value>,
    wander_pos: (i32, i32, i32),
    wandf: u32,
    provocative_sound: bool,
    hp: u16,
    special_attacks: HashMap<String, serde_json::Value>,
    friendly: i8,
    fish_population: u8,
    faction: String,
    mission_ids: Vec<serde_json::Value>,
    mission_fused: Vec<serde_json::Value>,
    no_extra_death_drops: bool,
    dead: bool,
    anger: i16,
    morale: i16,
    hallucination: bool,
    ammo: HashMap<String, i16>,
    underwater: bool,
    upgrades: bool,
    upgrade_time: i32,
    reproduces: bool,
    baby_timer: Option<serde_json::Value>,
    biosignatures: bool,
    biosig_timer: i32,
    udder_timer: u32,
    summon_time_limit: Option<serde_json::Value>,
    inv: Vec<serde_json::Value>,
    dragged_foe_id: i8,
    mounted_player_id: i8,
    dissectable_inv: Option<serde_json::Value>,
    lifespan_end: Option<serde_json::Value>,
    next_patrol_point: Option<serde_json::Value>,
    patrol_route: Option<serde_json::Value>,
    horde_attraction: Option<serde_json::Value>,
    battery_item: Option<serde_json::Value>,
}
