use crate::spawn::{SubzoneSpawner, VisibleRegion, ZoneSpawner};
use crate::{
    DespawnSubzoneLevel, DespawnZoneLevel, Expanded, Explored, Focus, MissingAsset, Region,
    SpawnSubzoneLevel, SpawnZoneLevel, TileSpawner, UpdateZoneLevelVisibility, VisualizationUpdate,
    ZoneLevelIds, ZoneRegion,
};
use bevy::ecs::{schedule::ScheduleConfigs, system::ScheduleSystem};
use bevy::platform::collections::HashSet;
use bevy::prelude::{
    Added, AssetEvent, Assets, Children, Commands, Entity, GlobalTransform,
    IntoScheduleConfigs as _, MessageReader, MessageWriter, Query, RelationshipTarget as _, Res,
    ResMut, Visibility, With, debug, on_message, warn,
};
use gameplay_cdda::{
    Exploration, MapAsset, MapManager, MapMemoryAsset, MapMemoryManager, OvermapAsset,
    OvermapBufferAsset, OvermapBufferManager, OvermapManager,
};
use gameplay_cdda_active_sav::ActiveSav;
use gameplay_local::GameplayLocal;
use gameplay_location::{
    Level, Pos, SubzoneLevel, SubzoneLevelCache, VisionDistance, Zone, ZoneLevel, ZoneLevelCache,
};
use std::cmp::Ordering;
use std::time::{Duration, Instant};
use util::{MessageBuffer, log_if_slow};

const MAX_EXPAND_DISTANCE: i32 = 10;

pub(crate) fn handle_region_asset_events() -> ScheduleConfigs<ScheduleSystem> {
    (
        (
            (
                handle_overmap_buffer_events.run_if(on_message::<AssetEvent<OvermapBufferAsset>>),
                handle_overmap_events.run_if(on_message::<AssetEvent<OvermapAsset>>),
            ),
            update_zone_levels_with_missing_assets
                .run_if(on_message::<AssetEvent<OvermapBufferAsset>>),
        )
            .chain(),
        handle_map_events.run_if(on_message::<AssetEvent<MapAsset>>),
        handle_map_memory_events.run_if(on_message::<AssetEvent<MapMemoryAsset>>),
    )
        .into_configs()
}

pub(crate) fn handle_zone_levels() -> ScheduleConfigs<ScheduleSystem> {
    (
        update_zone_levels,
        (
            spawn_zone_levels.run_if(on_message::<SpawnZoneLevel>),
            update_zone_level_visibility.run_if(on_message::<UpdateZoneLevelVisibility>),
        ),
    )
        .chain()
        .into_configs()
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn spawn_subzones_for_camera(
    mut spawn_subzone_level_writer: MessageWriter<SpawnSubzoneLevel>,
    mut despawn_subzone_level_writer: MessageWriter<DespawnSubzoneLevel>,
    focus: Focus,
    visible_region: VisibleRegion,
    subzone_level_cache: Res<SubzoneLevelCache>,
    mut previous_camera_global_transform: GameplayLocal<GlobalTransform>,
    mut expanded: ResMut<Expanded>,
    subzone_levels: Query<&SubzoneLevel>,
) {
    let start = Instant::now();

    // TODO fix respawning expanded subzones after loading a save game twice, because the Local resources might not change

    if visible_region.global_transform() == *previous_camera_global_transform.get() {
        return;
    }

    let minimal_expanded_zones = zones_in_sight_distance(Pos::from(&focus));
    let maximal_expanded_zones = maximal_expanded_zones(Zone::from(Pos::from(&focus)));
    let expanded_region = visible_region
        .calculate_all()
        .clamp(&minimal_expanded_zones, &maximal_expanded_zones);
    if !expanded.update(expanded_region) {
        return;
    }

    spawn_expanded_subzone_levels(
        &mut spawn_subzone_level_writer,
        &subzone_level_cache,
        &expanded.region,
    );
    despawn_expanded_subzone_levels(
        &mut despawn_subzone_level_writer,
        &subzone_levels,
        &expanded.region,
    );

    *previous_camera_global_transform.get() = visible_region.global_transform();

    log_if_slow("spawn_subzones_for_camera", start);
}

fn zones_in_sight_distance(focus_pos: Pos) -> Region {
    let from = Zone::from(focus_pos.horizontal_offset(
        -VisionDistance::MAX_VISION_TILES,
        -VisionDistance::MAX_VISION_TILES,
    ));
    let to = Zone::from(focus_pos.horizontal_offset(
        VisionDistance::MAX_VISION_TILES,
        VisionDistance::MAX_VISION_TILES,
    ));
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

fn spawn_expanded_subzone_levels(
    spawn_subzone_level_writer: &mut MessageWriter<SpawnSubzoneLevel>,
    subzone_level_cache: &SubzoneLevelCache,
    expanded_region: &Region,
) {
    for zone_level in expanded_region.zone_levels() {
        for subzone_level in zone_level.subzone_levels() {
            let missing = subzone_level_cache.get(subzone_level).is_none();
            if missing {
                spawn_subzone_level_writer.write(SpawnSubzoneLevel { subzone_level });
            }
        }
    }
}

fn despawn_expanded_subzone_levels(
    despawn_subzone_level_writer: &mut MessageWriter<DespawnSubzoneLevel>,
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
            despawn_subzone_level_writer.write(DespawnSubzoneLevel { subzone_level });
        });
}

pub(crate) fn spawn_subzone_levels(
    mut spawn_subzone_level_buffer: MessageBuffer<SpawnSubzoneLevel>,
    mut subzone_spawner: SubzoneSpawner,
    mut map_manager: MapManager,
    mut map_memory_manager: MapMemoryManager,
    mut overmap_buffer_manager: OvermapBufferManager,
    mut vizualization_update: ResMut<VisualizationUpdate>,
) {
    let start = Instant::now();
    if spawn_subzone_level_buffer.is_empty() {
        return;
    }

    *vizualization_update = VisualizationUpdate::Forced;

    // Prevent duplicates
    let mut added = HashSet::new();

    spawn_subzone_level_buffer.handle(
        |spawn_event| {
            if added.insert(spawn_event.subzone_level) {
                subzone_spawner.spawn_subzone_level(
                    &mut map_manager,
                    &mut map_memory_manager,
                    &mut overmap_buffer_manager,
                    spawn_event.subzone_level,
                );
            }
        },
        Duration::from_millis(3),
    );

    log_if_slow("spawn_subzone_levels", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_zone_levels(
    mut spawn_zone_level_writer: MessageWriter<SpawnZoneLevel>,
    mut update_zone_level_visibility_writer: MessageWriter<UpdateZoneLevelVisibility>,
    mut despawn_zone_level_writer: MessageWriter<DespawnZoneLevel>,
    focus: Focus,
    visible_region: VisibleRegion,
    zone_level_entities: Res<ZoneLevelCache>,
    mut previous_camera_global_transform: GameplayLocal<GlobalTransform>,
    mut previous_visible_region: GameplayLocal<Region>,
    zone_levels: Query<(Entity, &ZoneLevel, &Children), With<Visibility>>,
    new_subzone_levels: Query<(), Added<SubzoneLevel>>,
) {
    // Zone level visibility: not SeenFrom::Never and not open sky, deep rock, etc.
    // Zone level child visibility: not expanded, even when zoomed out

    let start = Instant::now();

    //trace!(
    //    "update_zone_levels {:?} {:?}",
    //    new_subzone_levels.iter().collect::<Vec<_>>().len(),
    //    new_subzone_levels.is_empty()
    //);

    let global_transform = visible_region.global_transform();
    if global_transform == *previous_camera_global_transform.get() && new_subzone_levels.is_empty()
    {
        return;
    }

    // Zone levels above zero add little value, so we always skip these.
    let visible_region = visible_region.calculate_ground();
    //trace!("Visible region: {:?}", &visible_region);
    if visible_region == *previous_visible_region.get() && new_subzone_levels.is_empty() {
        return;
    }
    //trace!("update_zone_levels refresh");
    //trace!("{:?}", (&visible_region);

    let shown_level = if Level::from(&focus).compare_to_ground() == Ordering::Less {
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
            spawn_zone_level_writer.write(SpawnZoneLevel { zone_level });
        }
    }

    for (entity, &zone_level, children) in zone_levels.iter() {
        if visible_region.contains_zone_level(zone_level) {
            update_zone_level_visibility_writer.write(UpdateZoneLevelVisibility {
                zone_level,
                children: children.iter().collect(),
            });
        } else {
            despawn_zone_level_writer.write(DespawnZoneLevel { entity });
        }
    }

    *previous_camera_global_transform.get() = global_transform;
    *previous_visible_region.get() = visible_region;

    log_if_slow("update_zone_levels", start);
}

fn spawn_zone_levels(
    mut spawn_zone_level_reader: MessageReader<SpawnZoneLevel>,
    mut zone_spawner: ZoneSpawner,
) {
    let start = Instant::now();

    zone_spawner.spawn_zone_levels(&mut spawn_zone_level_reader);

    log_if_slow("spawn_zone_levels", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_zone_level_visibility(
    mut commands: Commands,
    mut update_zone_level_visibility_reader: MessageReader<UpdateZoneLevelVisibility>,
    focus: Focus,
    visible_region: VisibleRegion,
    explored: Res<Explored>,
) {
    let start = Instant::now();

    debug!(
        "Updating the visibility of {} zone levels",
        update_zone_level_visibility_reader.len()
    );

    let visible_region = visible_region.calculate_ground();

    for update_zone_level_visibility_event in update_zone_level_visibility_reader.read() {
        let visibility = explored.zone_level_visibility(
            &focus,
            update_zone_level_visibility_event.zone_level,
            &visible_region,
        );
        for &entity in &update_zone_level_visibility_event.children {
            // Removing 'Visibility' and 'ComputedVisibility' is not more performant in Bevy 0.9
            commands.entity(entity).insert(visibility);
        }
    }

    log_if_slow("update_zone_level_visibility", start);
}

fn handle_map_events(
    mut spawn_subzone_level_writer: MessageWriter<SpawnSubzoneLevel>,
    mut map_manager: MapManager,
) {
    let start = Instant::now();

    spawn_subzone_level_writer.write_batch(
        map_manager
            .read_loaded_assets()
            .flat_map(|zone_level| zone_level.subzone_levels())
            .map(|subzone_level| SpawnSubzoneLevel { subzone_level }),
    );

    log_if_slow("handle_map_events", start);
}

#[expect(clippy::needless_pass_by_value)]
fn handle_map_memory_events(
    mut explorations: MessageWriter<Exploration>,
    mut spawn_subzone_level_writer: MessageWriter<SpawnSubzoneLevel>,
    subzone_level_cache: Res<SubzoneLevelCache>,
    expanded: Res<Expanded>,
    mut map_memory_manager: MapMemoryManager,
) {
    let start = Instant::now();

    explorations.write_batch(map_memory_manager.read_seen_pos());

    spawn_expanded_subzone_levels(
        &mut spawn_subzone_level_writer,
        &subzone_level_cache,
        &expanded.region,
    );

    log_if_slow("handle_map_memory_events", start);
}

fn handle_overmap_buffer_events(
    mut explorations: MessageWriter<Exploration>,
    mut overmap_buffer_manager: OvermapBufferManager,
) {
    let start = Instant::now();

    explorations.write_batch(overmap_buffer_manager.read_seen_zone_levels());

    log_if_slow("handle_overmap_buffer_events", start);
}

#[expect(clippy::needless_pass_by_value)]
fn handle_overmap_events(
    mut overmap_events: MessageReader<AssetEvent<OvermapAsset>>,
    overmap_assets: Res<Assets<OvermapAsset>>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
    overmap_manager: OvermapManager,
) {
    let start = Instant::now();

    for overmap_asset_event in overmap_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = overmap_asset_event {
            let Some(overzone) = overmap_manager.overzone(id) else {
                // This may be an asset of a previous gameplay.
                warn!("Unknown overmap asset {id:?} loaded");
                continue;
            };

            let overmap = overmap_assets.get(*id).expect("Overmap loaded");
            zone_level_ids.load(overzone, overmap);
        }
    }

    log_if_slow("handle_overmap_events", start);
}

fn update_zone_levels_with_missing_assets(
    mut zone_spawner: ZoneSpawner,
    zone_levels: Query<(Entity, &ZoneLevel), With<MissingAsset>>,
) {
    let start = Instant::now();

    if zone_levels.iter().len() == 0 {
        return;
    }

    zone_spawner.complete_missing_assets(zone_levels);

    log_if_slow("update_zone_levels_with_missing_assets", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn spawn_initial_entities(active_sav: Res<ActiveSav>, mut spawner: TileSpawner) {
    spawner.spawn_light();

    let sav = active_sav.sav();
    let spawn_pos = Zone {
        x: i32::from(sav.om_x) * 180 + i32::from(sav.levx) / 2,
        z: i32::from(sav.om_y) * 180 + i32::from(sav.levy) / 2,
    }
    .zone_level(Level::new(sav.levz))
    .base_corner()
    .horizontal_offset(
        12 * i32::from(sav.levx % 2) + 24,
        12 * i32::from(sav.levy % 2) + 24,
    );
    spawner.spawn_characters(spawn_pos);
}

pub(crate) fn update_explored(
    mut explorations: MessageReader<Exploration>,
    mut explored: ResMut<Explored>,
) {
    let start = Instant::now();

    explored.add(explorations.read());

    log_if_slow("update_explored", start);
}
