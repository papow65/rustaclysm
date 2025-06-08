use crate::{
    CddaAmount, CharacterInfo, FlatVec, InfoId, OvermapTerrainInfo, RepetitionBlock,
    RequiredLinkedLater,
};
use bevy_platform::collections::HashMap;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::sync::{Arc, OnceLock};

/// Corresponds to an 'overmap' in CDDA. It defines the layout of 180x180 `Zone`s.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overmap {
    pub layers: [OvermapLevel; Self::LEVEL_AMOUNT],
    pub region_id: JsonValue,
    pub monster_groups: JsonValue,
    pub cities: JsonValue,
    pub connections_out: JsonValue,
    pub radios: JsonValue,
    pub monster_map: FlatVec<(SubzoneOffset, Monster), 2>,
    pub tracked_vehicles: JsonValue,
    pub scent_traces: JsonValue,
    pub npcs: JsonValue,
    pub camps: JsonValue,
    pub overmap_special_placements: JsonValue,
    pub mapgen_arg_storage: Option<JsonValue>,
    pub mapgen_arg_index: Option<JsonValue>,
    pub joins_used: Option<JsonValue>,
    pub predecessors: Option<JsonValue>,

    /// Marker to remember the state of the links
    #[serde(skip)]
    pub linked: OnceLock<()>,
}

impl Overmap {
    pub const LEVEL_AMOUNT: usize = 21;
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OvermapLevel(pub RepetitionBlock<InfoId<OvermapTerrainInfo>>);

impl OvermapLevel {
    #[must_use]
    pub fn all(id: InfoId<OvermapTerrainInfo>) -> Self {
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
    effects: HashMap<Arc<str>, JsonValue>,
    damage_over_time_map: Vec<JsonValue>,
    values: HashMap<Arc<str>, JsonValue>,
    blocks_left: u8,
    dodges_left: u8,
    num_blocks_bonus: u8,
    num_dodges_bonus: u8,
    armor_bash_bonus: Option<u8>,
    armor_cut_bonus: Option<u8>,
    armor_bullet_bonus: Option<u8>,
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
    body: HashMap<Arc<str>, JsonValue>,

    #[serde(rename = "typeid")]
    pub info: RequiredLinkedLater<CharacterInfo>,

    unique_name: Arc<str>,
    nickname: Arc<str>,
    goal: Option<JsonValue>,
    wander_pos: (i32, i32, i32),
    wandf: u32,
    provocative_sound: bool,
    hp: u16,
    special_attacks: HashMap<Arc<str>, JsonValue>,
    friendly: i8,
    fish_population: u8,
    faction: Arc<str>,
    mission_ids: Vec<JsonValue>,
    mission_fused: Vec<JsonValue>,
    no_extra_death_drops: bool,
    dead: bool,
    anger: i16,
    morale: i16,
    hallucination: bool,
    ammo: HashMap<Arc<str>, i16>,
    underwater: bool,
    upgrades: bool,
    upgrade_time: i32,
    reproduces: bool,
    baby_timer: Option<JsonValue>,
    biosignatures: bool,
    biosig_timer: i32,
    udder_timer: u32,
    summon_time_limit: Option<JsonValue>,
    inv: Vec<JsonValue>,
    dragged_foe_id: i8,
    mounted_player_id: i8,
    dissectable_inv: Option<JsonValue>,
    lifespan_end: Option<JsonValue>,
    next_patrol_point: Option<JsonValue>,
    patrol_route: Option<JsonValue>,
    horde_attraction: Option<JsonValue>,
    battery_item: Option<JsonValue>,
    aggro_character: Option<JsonValue>,
    armor_bonus: Option<JsonValue>,
    grabbed_limbs: Option<JsonValue>,
}
