use crate::prelude::*;
use bevy::{
    asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset},
    reflect::{TypePath, TypeUuid},
    utils::HashMap,
};
use serde::Deserialize;
use std::{str::from_utf8, sync::OnceLock};

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
#[derive(Debug, Deserialize, TypePath, TypeUuid)]
#[serde(deny_unknown_fields)]
#[type_path = "cdda::world::Overmap"]
#[uuid = "a4067c84-4c64-4765-9000-53a045919796"]
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

/** This loads both overmaps and overmap buffers, since those have the same extensions. */
#[derive(Default)]
pub(crate) struct OvermapLoader;

impl OvermapLoader {
    const EXTENSION_MAX: usize = 1000;
    const EXTENSION_COUNT: usize = 2 * Self::EXTENSION_MAX + 1;
}

impl AssetLoader for OvermapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let newline_pos = bytes
                .windows(1)
                .position(|window| window == b"\n")
                .expect("Version line");
            let after_first_line = bytes.split_at(newline_pos).1;

            let file_name = load_context
                .path()
                .file_name()
                .expect("File name present")
                .to_str()
                .expect("Unicode filename");

            if file_name.starts_with("o.") {
                let overmap_result = serde_json::from_slice::<Overmap>(after_first_line);
                let overmap = overmap_result.map_err(|e| {
                    eprintln!(
                        "Overmap loading error: {file_name:?} {:?} {e:?}",
                        from_utf8(&bytes[0..40])
                    );
                    e
                })?;
                load_context.set_default_asset(LoadedAsset::new(overmap));
            } else {
                let overmap_buffer_result =
                    serde_json::from_slice::<OvermapBuffer>(after_first_line);
                let overmap_buffer = overmap_buffer_result.map_err(|e| {
                    eprintln!(
                        "Overmap buffer loading error: {file_name:?} {:?} {e:?}",
                        from_utf8(&bytes[0..40])
                    );
                    e
                })?;
                load_context.set_default_asset(LoadedAsset::new(overmap_buffer));
            }
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static STRINGS: OnceLock<[String; OvermapLoader::EXTENSION_COUNT]> = OnceLock::new();
        static EXTENSIONS: OnceLock<[&str; OvermapLoader::EXTENSION_COUNT]> = OnceLock::new();

        EXTENSIONS.get_or_init(|| {
            let strings = STRINGS.get_or_init(|| {
                let mut i = -(Self::EXTENSION_MAX as isize);
                [(); Self::EXTENSION_COUNT].map(|_| {
                    let string = i.to_string();
                    i += 1;
                    string
                })
            });

            let mut j = 0;
            [(); Self::EXTENSION_COUNT].map(|_| {
                let extension = strings[j].as_str();
                j += 1;
                extension
            })
        })
    }
}

#[cfg(test)]
mod overmap_buffer_tests {
    use super::*;
    #[test]
    fn check_extensions() {
        let extensions = OvermapLoader.extensions();
        assert_eq!(
            extensions.len(),
            OvermapLoader::EXTENSION_COUNT,
            "{extensions:?}"
        );
        assert_eq!(
            extensions[0],
            (-(OvermapLoader::EXTENSION_MAX as isize))
                .to_string()
                .as_str(),
            "{extensions:?}"
        );
        assert_eq!(
            extensions.last().expect("many items"),
            &OvermapLoader::EXTENSION_MAX.to_string().as_str(),
            "{extensions:?}"
        );
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
