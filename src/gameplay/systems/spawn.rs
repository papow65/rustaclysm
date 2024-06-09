use crate::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*};
use std::{cmp::Ordering, time::Instant};

const MAX_EXPAND_DISTANCE: i32 = 10;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_subzones_for_camera(
    mut spawn_subzone_level_writer: EventWriter<SpawnSubzoneLevel>,
    mut despawn_subzone_level_writer: EventWriter<DespawnSubzoneLevel>,
    focus: Focus,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut session: GameplaySession,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut expanded: ResMut<Expanded>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    subzone_levels: Query<&SubzoneLevel>,
) {
    let start = Instant::now();

    if session.is_changed() {
        *previous_camera_global_transform = GlobalTransform::default();
    }

    // TODO fix respawning expanded subzones after loading a save game twice, because the Local resources might not change

    let (camera, &global_transform) = cameras.single();
    if global_transform == *previous_camera_global_transform {
        return;
    }

    let expanded_region = expanded_region(&focus, camera, &global_transform);
    if !expanded.update(expanded_region) {
        return;
    }

    spawn_expanded_subzone_levels(
        &mut spawn_subzone_level_writer,
        &subzone_level_entities,
        &expanded.region,
    );
    despawn_expanded_subzone_levels(
        &mut despawn_subzone_level_writer,
        &subzone_levels,
        &expanded.region,
    );

    *previous_camera_global_transform = global_transform;

    log_if_slow("spawn_subzones_for_camera", start);
}

fn zones_in_sight_distance(focus_pos: Pos) -> Region {
    let from =
        Zone::from(focus_pos.horizontal_offset(-MAX_VISIBLE_DISTANCE, -MAX_VISIBLE_DISTANCE));
    let to = Zone::from(focus_pos.horizontal_offset(MAX_VISIBLE_DISTANCE, MAX_VISIBLE_DISTANCE));
    Region::from(&ZoneRegion::new(from.x..=to.x, from.z..=to.z))
}

/// Upper limit for expanding subzones
fn maximal_expanded_zones(player_zone: Zone) -> Region {
    let x_from = player_zone.x - MAX_EXPAND_DISTANCE;
    let x_to = player_zone.x + MAX_EXPAND_DISTANCE;
    let z_from = player_zone.z - MAX_EXPAND_DISTANCE;
    let z_to = player_zone.z + MAX_EXPAND_DISTANCE;

    Region::from(&ZoneRegion::new(x_from..=x_to, z_from..=z_to))
}

/// Region of expanded zones
fn expanded_region(focus: &Focus, camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let minimal_expanded_zones = zones_in_sight_distance(Pos::from(focus));
    let maximal_expanded_zones = maximal_expanded_zones(Zone::from(Pos::from(focus)));

    visible_region(camera, global_transform).clamp(&minimal_expanded_zones, &maximal_expanded_zones)
}

/// Region visible on the camera
fn visible_region(camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let Some(Rect {
        min: corner_min,
        max: corner_max,
    }) = camera.logical_viewport_rect()
    else {
        return Region::new(&Vec::new());
    };

    let mut zone_levels = Vec::new();
    let floor: fn(f32) -> f32 = f32::floor;
    let ceil: fn(f32) -> f32 = f32::ceil;
    for level in Level::ALL {
        for (corner, round_x, round_z) in [
            (Vec2::new(corner_min.x, corner_min.y), floor, floor),
            (Vec2::new(corner_min.x, corner_max.y), floor, ceil),
            (Vec2::new(corner_max.x, corner_min.y), ceil, floor),
            (Vec2::new(corner_max.x, corner_max.y), ceil, ceil),
        ] {
            let Some(ray) = camera.viewport_to_world(global_transform, corner) else {
                continue;
            };

            let ray_distance = (level.f32() - ray.origin.y) / ray.direction.y;
            // The camera only looks forward.
            if 0.0 < ray_distance {
                let floor = ray.get_point(ray_distance);
                //dbg!((level, ray_distance, floor.x, floor.z));
                zone_levels.push(ZoneLevel::from(Pos {
                    x: round_x(floor.x) as i32,
                    level,
                    z: round_z(floor.z) as i32,
                }));
            }
        }
    }

    Region::new(&zone_levels)
}

fn spawn_expanded_subzone_levels(
    spawn_subzone_level_writer: &mut EventWriter<SpawnSubzoneLevel>,
    subzone_level_entities: &SubzoneLevelEntities,
    expanded_region: &Region,
) {
    for zone_level in expanded_region.zone_levels() {
        for subzone_level in zone_level.subzone_levels() {
            let missing = subzone_level_entities.get(subzone_level).is_none();
            if missing {
                spawn_subzone_level_writer.send(SpawnSubzoneLevel { subzone_level });
            }
        }
    }
}

fn despawn_expanded_subzone_levels(
    despawn_subzone_level_writer: &mut EventWriter<DespawnSubzoneLevel>,
    subzone_levels: &Query<&SubzoneLevel>,
    expanded_region: &Region,
) {
    // we use hashmap keys to get rid of duplicates
    subzone_levels
        .iter()
        .filter(|subzone_level| {
            !expanded_region.contains_zone_level(ZoneLevel::from(**subzone_level))
        })
        .copied()
        .for_each(|subzone_level| {
            despawn_subzone_level_writer.send(DespawnSubzoneLevel { subzone_level });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_subzone_levels(
    mut spawn_subzone_level_reader: EventReader<SpawnSubzoneLevel>,
    mut subzone_spawner: SubzoneSpawner,
    mut map_manager: MapManager,
    mut map_memory_manager: MapMemoryManager,
) {
    let start = Instant::now();

    println!(
        "Spawning {} subzone levels",
        spawn_subzone_level_reader.len()
    );

    for spawn_event in spawn_subzone_level_reader.read() {
        subzone_spawner.spawn_subzone_level(
            &mut map_manager,
            &mut map_memory_manager,
            spawn_event.subzone_level,
        );
    }

    log_if_slow("spawn_subzone_levels", start);
}

/// This is an intentionally exclusive system to prevent an occasional panic.
/// See <https://bevyengine.org/learn/errors/b0003/>
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_subzone_levels(
    world: &mut World,
    sytem_state: &mut SystemState<(
        Commands,
        EventReader<DespawnSubzoneLevel>,
        ResMut<SubzoneLevelEntities>,
    )>,
) {
    let start = Instant::now();

    let (mut commands, mut despawn_subzone_level_reader, mut subzone_level_entities) =
        sytem_state.get_mut(world);

    println!(
        "Despawning {} subzone levels",
        despawn_subzone_level_reader.len()
    );

    for despawn_event in despawn_subzone_level_reader.read() {
        if let Some(entity) = subzone_level_entities.remove(despawn_event.subzone_level) {
            commands.entity(entity).despawn_recursive();
        }
    }

    sytem_state.apply(world);

    log_if_slow("despawn_subzone_levels", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_zone_levels(
    mut spawn_zone_level_writer: EventWriter<SpawnZoneLevel>,
    mut update_zone_level_visibility_writer: EventWriter<UpdateZoneLevelVisibility>,
    mut despawn_zone_level_writer: EventWriter<DespawnZoneLevel>,
    focus: Focus,
    zone_level_entities: Res<ZoneLevelEntities>,
    mut session: GameplaySession,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_visible_region: Local<Region>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    zone_levels: Query<(Entity, &ZoneLevel, &Children), With<Visibility>>,
    new_subzone_levels: Query<(), Added<SubzoneLevel>>,
) {
    // Zone level visibility: not SeenFrom::Never and not open sky, deep rock, etc.
    // Zone level child visibility: not expanded, even when zoomed out

    let start = Instant::now();

    if session.is_changed() {
        *previous_camera_global_transform = GlobalTransform::default();
        *previous_visible_region = Region::default();
    }

    //println!(
    //    "update_zone_levels {:?} {:?}",
    //    new_subzone_levels.iter().collect::<Vec<_>>().len(),
    //    new_subzone_levels.is_empty()
    //);

    let (camera, &global_transform) = cameras.single();
    if global_transform == *previous_camera_global_transform && new_subzone_levels.is_empty() {
        return;
    }

    // Zone levels above zero add little value, so we always skip these.
    let visible_region = visible_region(camera, &global_transform).ground_only();
    //println!("Visible region: {:?}", &visible_region);
    if visible_region == *previous_visible_region && new_subzone_levels.is_empty() {
        return;
    }
    //println!("update_zone_levels refresh");
    //dbg!(&visible_region);

    let shown_level = if let Ordering::Less = Level::from(&focus).compare_to_ground() {
        Level::from(&focus)
    } else {
        Level::ZERO
    };

    for zone_level in visible_region
        .zone_levels()
        .into_iter()
        .filter(|zone_level| zone_level.level == shown_level)
    {
        if zone_level_entities.get(zone_level).is_none() {
            spawn_zone_level_writer.send(SpawnZoneLevel { zone_level });
        }
    }

    for (entity, &zone_level, children) in zone_levels.iter() {
        if visible_region.contains_zone_level(zone_level) {
            update_zone_level_visibility_writer.send(UpdateZoneLevelVisibility {
                zone_level,
                children: children.iter().copied().collect(),
            });
        } else {
            despawn_zone_level_writer.send(DespawnZoneLevel { entity });
        }
    }

    *previous_camera_global_transform = global_transform;
    *previous_visible_region = visible_region;

    log_if_slow("update_zone_levels", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_zone_levels(
    mut spawn_zone_level_reader: EventReader<SpawnZoneLevel>,
    focus: Focus,
    mut zone_spawner: ZoneSpawner,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let start = Instant::now();

    println!("Spawning {} zone levels", spawn_zone_level_reader.len());

    let (camera, &global_transform) = cameras.single();
    let visible_region = visible_region(camera, &global_transform).ground_only();

    for spawn_event in spawn_zone_level_reader.read() {
        let visibility = zone_level_visibility(
            &mut zone_spawner,
            spawn_event.zone_level,
            &visible_region,
            &focus,
        );
        zone_spawner.spawn_zone_level(spawn_event.zone_level, &visibility);
    }

    log_if_slow("spawn_zone_levels", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_zone_level_visibility(
    mut commands: Commands,
    mut update_zone_level_visibility_reader: EventReader<UpdateZoneLevelVisibility>,
    focus: Focus,
    mut zone_spawner: ZoneSpawner,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let start = Instant::now();

    println!(
        "Updating the visibility of {} zone levels",
        update_zone_level_visibility_reader.len()
    );

    let (camera, &global_transform) = cameras.single();
    let visible_region = visible_region(camera, &global_transform).ground_only();

    for update_zone_level_visibility_event in update_zone_level_visibility_reader.read() {
        let visibility = zone_level_visibility(
            &mut zone_spawner,
            update_zone_level_visibility_event.zone_level,
            &visible_region,
            &focus,
        );
        for &entity in &update_zone_level_visibility_event.children {
            // Removing 'Visibility' and 'ComputedVisibility' is not more performant in Bevy 0.9
            commands.entity(entity).insert(visibility);
        }
    }

    log_if_slow("update_zone_level_visibility", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_zone_level(
    mut commands: Commands,
    mut despawn_zone_level_reader: EventReader<DespawnZoneLevel>,
    mut zone_level_entities: ResMut<ZoneLevelEntities>,
) {
    let start = Instant::now();

    println!("Despawning {} zone levels", despawn_zone_level_reader.len());

    for despawn_zone_level_event in despawn_zone_level_reader.read() {
        let entity = despawn_zone_level_event.entity;
        commands.entity(entity).despawn_recursive();
        zone_level_entities.remove(entity);
    }

    log_if_slow("despawn_zone_level", start);
}

fn zone_level_visibility(
    zone_spawner: &mut ZoneSpawner,
    zone_level: ZoneLevel,
    visible_region: &Region,
    focus: &Focus,
) -> Visibility {
    if zone_level.level == Level::from(focus).min(Level::ZERO)
        && zone_level.subzone_levels().iter().all(|subzone_level| {
            visible_region.contains_zone_level(ZoneLevel::from(*subzone_level))
                && zone_spawner
                    .spawner
                    .explored
                    .has_zone_level_been_seen(&mut zone_spawner.overmap_buffer_manager, zone_level)
                    .is_some_and(|seen_from| seen_from != SeenFrom::Never)
        })
    {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_map_events(
    mut map_asset_events: EventReader<AssetEvent<Map>>,
    mut subzone_spawner: SubzoneSpawner,
    mut map_manager: MapManager,
    mut map_memory_manager: MapMemoryManager,
) {
    for map_asset_event in map_asset_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = map_asset_event {
            let zone_level = map_manager
                .zone_level(id)
                .unwrap_or_else(|| panic!("{id:?} shoould be a known map asset id"));

            for subzone_level in zone_level.subzone_levels() {
                if subzone_spawner
                    .subzone_level_entities
                    .get(subzone_level)
                    .is_none()
                {
                    subzone_spawner.spawn_subzone_level(
                        &mut map_manager,
                        &mut map_memory_manager,
                        subzone_level,
                    );
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_map_memory_events(
    mut map_memory_asset_events: EventReader<AssetEvent<MapMemory>>,
    mut spawn_subzone_level_writer: EventWriter<SpawnSubzoneLevel>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    expanded: Res<Expanded>,
    mut explored: ResMut<Explored>,
    mut map_memory_manager: MapMemoryManager,
) {
    for map_asset_event in map_memory_asset_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = map_asset_event {
            let base_zone_level = map_memory_manager
                .base_zone_level(id)
                .expect("Map memory known");
            //println!("Loading map memory for {base_zone_level:?}");
            explored.load_memory(&mut map_memory_manager, base_zone_level);
        }
    }

    spawn_expanded_subzone_levels(
        &mut spawn_subzone_level_writer,
        &subzone_level_entities,
        &expanded.region,
    );
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_overmap_buffer_events(
    mut overmap_buffer_events: EventReader<AssetEvent<OvermapBuffer>>,
    overmap_buffer_assets: Res<Assets<OvermapBuffer>>,
    mut explored: ResMut<Explored>,
    mut overmap_buffer_manager: OvermapBufferManager,
) {
    let start = Instant::now();

    for overmap_asset_event in overmap_buffer_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = overmap_asset_event {
            if let Some(overzone) = overmap_buffer_manager.overzone(id) {
                let overmap_buffer = overmap_buffer_assets
                    .get(*id)
                    .expect("Overmap buffer loaded");
                explored.load_buffer(overzone, overmap_buffer);
            }
        }
    }

    log_if_slow("update_zone_level_visibility", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_overmap_events(
    mut overmap_events: EventReader<AssetEvent<Overmap>>,
    overmap_assets: Res<Assets<Overmap>>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
    mut overmap_manager: OvermapManager,
) {
    let start = Instant::now();

    for overmap_asset_event in overmap_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = overmap_asset_event {
            if let Some(overzone) = overmap_manager.overzone(id) {
                let overmap = overmap_assets.get(*id).expect("Overmap loaded");
                zone_level_ids.load(overzone, overmap);
            }
        }
    }

    log_if_slow("update_zone_level_visibility", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_zone_levels_with_missing_assets(
    focus: Focus,
    mut zone_spawner: ZoneSpawner,
    zone_levels: Query<(Entity, &ZoneLevel), With<MissingAsset>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let start = Instant::now();

    if zone_levels.iter().len() == 0 {
        return;
    }

    let (camera, &global_transform) = cameras.single();
    let visible_region = visible_region(camera, &global_transform).ground_only();

    for (entity, &zone_level) in &zone_levels {
        let Some(seen_from) = zone_spawner
            .spawner
            .explored
            .has_zone_level_been_seen(&mut zone_spawner.overmap_buffer_manager, zone_level)
        else {
            continue;
        };

        let Some(definition) = zone_spawner
            .zone_level_ids
            .get(&mut zone_spawner.overmap_manager, zone_level)
            .map(|object_id| ObjectDefinition {
                category: ObjectCategory::ZoneLevel,
                id: object_id.clone(),
            })
        else {
            continue;
        };

        let child_visibility =
            zone_level_visibility(&mut zone_spawner, zone_level, &visible_region, &focus);

        zone_spawner.complete_zone_level(
            entity,
            zone_level,
            seen_from,
            &definition,
            &child_visibility,
        );
        zone_spawner
            .spawner
            .commands
            .entity(entity)
            .remove::<MissingAsset>();
    }

    log_if_slow("update_zone_level_visibility", start);
}
