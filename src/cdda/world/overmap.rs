use crate::prelude::{
    CddaAmount, FlatVec, Level, ObjectId, Overzone, PathFor, RepetitionBlock, WorldPath,
};
use bevy::utils::HashMap;
use serde::Deserialize;
use std::fs::read_to_string;

pub(crate) type OvermapPath = PathFor<Overmap>;

impl OvermapPath {
    pub(crate) fn new(world_path: &WorldPath, overzone: Overzone) -> Self {
        Self::init(
            world_path
                .0
                .join(format!("o.{}.{}", overzone.x, overzone.z)),
        )
    }
}

/** Corresponds to an 'overmap' in CDDA. It defines the layout of 180x180 `Zone`s. */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Overmap {
    pub(crate) layers: [OvermapLevel; Level::AMOUNT],

    #[allow(unused)] // TODO
    region_id: serde_json::Value,

    #[allow(unused)] // TODO
    monster_groups: serde_json::Value,

    #[allow(unused)] // TODO
    cities: serde_json::Value,

    #[allow(unused)] // TODO
    connections_out: serde_json::Value,

    #[allow(unused)] // TODO
    radios: serde_json::Value,

    #[allow(unused)] // TODO
    monster_map: FlatVec<(SubzoneOffset, Monster), 2>,

    #[allow(unused)] // TODO
    tracked_vehicles: serde_json::Value,

    #[allow(unused)] // TODO
    scent_traces: serde_json::Value,

    #[allow(unused)] // TODO
    npcs: serde_json::Value,

    #[allow(unused)] // TODO
    camps: serde_json::Value,

    #[allow(unused)] // TODO
    overmap_special_placements: serde_json::Value,

    #[allow(unused)] // TODO
    mapgen_arg_storage: Option<serde_json::Value>,

    #[allow(unused)] // TODO
    mapgen_arg_index: Option<serde_json::Value>,

    #[allow(unused)] // TODO
    joins_used: Option<serde_json::Value>,

    #[allow(unused)] // TODO
    predecessors: Option<serde_json::Value>,
}

impl Overmap {
    pub(crate) fn fallback() -> Self {
        Self {
            layers: [
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("empty_rock")),
                OvermapLevel::all(ObjectId::new("empty_rock")),
                OvermapLevel::all(ObjectId::new("solid_earth")),
                OvermapLevel::all(ObjectId::new("field")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
            ],
            region_id: serde_json::Value::Null,
            monster_groups: serde_json::Value::Null,
            cities: serde_json::Value::Null,
            connections_out: serde_json::Value::Null,
            radios: serde_json::Value::Null,
            monster_map: FlatVec(Vec::new()),
            tracked_vehicles: serde_json::Value::Null,
            scent_traces: serde_json::Value::Null,
            npcs: serde_json::Value::Null,
            camps: serde_json::Value::Null,
            overmap_special_placements: serde_json::Value::Null,
            mapgen_arg_storage: None,
            mapgen_arg_index: None,
            joins_used: None,
            predecessors: None,
        }
    }
}

impl TryFrom<&OvermapPath> for Overmap {
    type Error = ();
    fn try_from(overmap_path: &OvermapPath) -> Result<Self, ()> {
        read_to_string(&overmap_path.0)
            .ok()
            .map(|s| {
                let first_newline = s.find('\n').unwrap();
                let after_first_line = s.split_at(first_newline).1;
                serde_json::from_str(after_first_line).unwrap_or_else(|err| {
                    panic!("Failed to deserialize overmap at {overmap_path:?}: {err:?}")
                })
            })
            .ok_or(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OvermapLevel(pub(crate) RepetitionBlock<ObjectId>);

impl OvermapLevel {
    fn all(id: ObjectId) -> Self {
        Self(RepetitionBlock::new(CddaAmount {
            obj: id,
            amount: 180 * 180,
        }))
    }
}

/** Offset of the subzone from the overmap */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SubzoneOffset(u16, u16, i8);

#[allow(unused)]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Monster {
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
    speed_bonus: i8,
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
    typeid: String,
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
    morale: i8,
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
}
