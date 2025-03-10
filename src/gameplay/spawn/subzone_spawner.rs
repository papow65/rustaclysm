use crate::application::ApplicationState;
use crate::gameplay::{
    AssetState, Infos, LevelOffset, LocalTerrain, MapManager, MapMemoryManager,
    OvermapBufferManager, OvermapManager, Overzone, PosOffset, RepetitionBlockExt as _,
    SubzoneLevel, SubzoneLevelEntities, ZoneLevel, ZoneLevelIds, spawn::TileSpawner,
    spawn::log_spawn_result,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Res, ResMut, StateScoped, Transform, Visibility};
use cdda_json_files::{
    CddaAmount, FlatVec, ObjectId, RepetitionBlock, RequiredLinkedLater, Submap, SubzoneOffset,
};
use std::sync::OnceLock;

#[derive(SystemParam)]
pub(crate) struct SubzoneSpawner<'w, 's> {
    infos: Res<'w, Infos>,
    zone_level_ids: ResMut<'w, ZoneLevelIds>,
    subzone_level_entities: ResMut<'w, SubzoneLevelEntities>,
    overmap_manager: OvermapManager<'w>,
    tile_spawner: TileSpawner<'w, 's>,
}

impl SubzoneSpawner<'_, '_> {
    pub(crate) fn spawn_subzone_level(
        &mut self,
        map_manager: &mut MapManager,
        map_memory_manager: &mut MapMemoryManager,
        overmap_buffer_manager: &mut OvermapBufferManager,
        subzone_level: SubzoneLevel,
    ) {
        if self.subzone_level_entities.get(subzone_level).is_some() {
            eprintln!("{subzone_level:?} already exists");
            return;
        }

        // Ensure all required assets
        let overzone = Overzone::from(ZoneLevel::from(subzone_level).zone);
        let asset_state = self.overmap_manager.load(overzone);
        if matches!(asset_state, AssetState::Nonexistent) {
            self.zone_level_ids.create_missing(overzone);
        }
        overmap_buffer_manager.load(overzone);

        match map_manager.submap(subzone_level) {
            AssetState::Available { asset: submap } => {
                match map_memory_manager.submap(subzone_level) {
                    AssetState::Available { .. } | AssetState::Nonexistent => {
                        self.spawn_submap(submap, subzone_level);
                    }
                    AssetState::Loading => {
                        // It will be spawned when it's fully loaded.
                    }
                }
            }
            AssetState::Loading => {
                // It will be spawned when it's fully loaded.
            }
            AssetState::Nonexistent => {
                if let Some(object_id) = self.zone_level_ids.get(ZoneLevel::from(subzone_level)) {
                    let submap = Self::fallback_submap(subzone_level, object_id);
                    self.spawn_submap(&submap, subzone_level);
                }
            }
        }
    }

    fn spawn_submap(&mut self, submap: &Submap, subzone_level: SubzoneLevel) {
        //println!("{:?} started...", subzone_level);
        let subzone_level_entity = self
            .tile_spawner
            .commands
            .spawn((
                Transform::IDENTITY,
                Visibility::Inherited,
                subzone_level,
                StateScoped(ApplicationState::Gameplay),
            ))
            .id();

        if submap.terrain.is_significant() {
            self.infos.link_submap(submap);

            let terrain = submap.terrain.load_as_subzone(subzone_level);
            let base_pos = subzone_level.base_corner();

            for x in 0..12 {
                for z in 0..12 {
                    let pos_offset = PosOffset {
                        x,
                        level: LevelOffset::ZERO,
                        z,
                    };
                    let pos = base_pos.horizontal_offset(x, z);
                    //dbg!("{pos:?}");
                    let Some(local_terrain) = LocalTerrain::at(&terrain, pos) else {
                        return;
                    };
                    let furniture_ids = submap
                        .furniture
                        .iter()
                        .filter_map(|at| pos_offset.get(at))
                        .filter_map(|required| {
                            required.get_or(|error| {
                                dbg!(error);
                            })
                        });
                    let item_repetitions =
                        submap.items.0.iter().filter_map(|at| pos_offset.get(at));
                    let spawns = submap
                        .spawns
                        .iter()
                        .filter(|spawn| spawn.x == pos_offset.x && spawn.z == pos_offset.z);
                    let fields = submap.fields.0.iter().filter_map(|at| pos_offset.get(at));
                    self.tile_spawner.spawn_tile(
                        &self.infos,
                        subzone_level_entity,
                        pos,
                        &local_terrain,
                        furniture_ids,
                        item_repetitions,
                        spawns,
                        fields,
                    );
                }
            }

            for vehicle in &submap.vehicles {
                let vehicle_pos = base_pos.horizontal_offset(vehicle.posx, vehicle.posy);
                let vehicle_entity =
                    self.tile_spawner
                        .spawn_vehicle(subzone_level_entity, vehicle_pos, vehicle);

                for vehicle_part in &vehicle.parts {
                    self.tile_spawner.spawn_vehicle_part(
                        &self.infos,
                        vehicle_entity,
                        vehicle_pos,
                        vehicle_part,
                    );
                }
            }

            if let AssetState::Available { asset: overmap } = self
                .overmap_manager
                .load(Overzone::from(ZoneLevel::from(subzone_level).zone))
            {
                let spawned_offset = SubzoneOffset::from(subzone_level);
                for monster in overmap
                    .0
                    .monster_map
                    .0
                    .iter()
                    .filter(|(at, _)| at == &spawned_offset)
                    .map(|(_, monster)| monster)
                {
                    log_spawn_result(self.tile_spawner.spawn_character(
                        &self.infos,
                        subzone_level_entity,
                        base_pos,
                        &monster.typeid,
                        None,
                    ));
                }
            }

            //println!("{:?} done", subzone_level);
        }

        self.subzone_level_entities
            .add(subzone_level, subzone_level_entity);
    }

    fn fallback_submap(subzone_level: SubzoneLevel, zone_object_id: &ObjectId) -> Submap {
        let terrain_id = ObjectId::new(if zone_object_id == &ObjectId::new("open_air") {
            "t_open_air"
        } else if zone_object_id == &ObjectId::new("solid_earth") {
            "t_soil"
        } else if [ObjectId::new("empty_rock"), ObjectId::new("deep_rock")].contains(zone_object_id)
        {
            "t_rock"
        } else if zone_object_id.is_moving_deep_water_zone() {
            "t_water_moving_dp"
        } else if zone_object_id.is_still_deep_water_zone() {
            "t_water_dp"
        } else if zone_object_id.is_grassy_zone() {
            "t_grass"
        } else if zone_object_id.is_road_zone() {
            "t_pavement"
        } else {
            "t_dirt"
        });
        Submap {
            version: 0,
            turn_last_touched: 0,
            coordinates: subzone_level.coordinates(),
            temperature: 0,
            radiation: Vec::new(),
            terrain: RepetitionBlock::new(CddaAmount {
                obj: RequiredLinkedLater::from(terrain_id),
                amount: 144,
            }),
            furniture: Vec::new(),
            items: FlatVec(Vec::new()),
            traps: Vec::new(),
            fields: FlatVec(Vec::new()),
            cosmetics: Vec::new(),
            spawns: Vec::new(),
            vehicles: Vec::new(),
            partial_constructions: Vec::new(),
            computers: Vec::new(),
            linked: OnceLock::default(),
        }
    }
}
