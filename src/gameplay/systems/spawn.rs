use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use std::time::Instant;

const MAX_EXPAND_DISTANCE: i32 = 10;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_subzones_for_camera(
    mut spawn_subzone_level_writer: EventWriter<SpawnSubzoneLevel>,
    mut collaspe_zone_level_writer: EventWriter<CollapseZoneLevel>,
    player_action_state: Res<PlayerActionState>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_expanded_region: Local<Region>,
    players: Query<&Pos, With<Player>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    subzone_levels: Query<&SubzoneLevel>,
) {
    let start = Instant::now();

    // TODO fix respawning expanded subzones after loading a save game twice, because the Local resources might not change

    let (camera, &global_transform) = cameras.single();
    if global_transform == *previous_camera_global_transform {
        return;
    }

    let &player_pos = players.single();
    let focus = Focus::new(&player_action_state, player_pos);
    let expanded_region = expanded_region(&focus, camera, &global_transform);
    if expanded_region == *previous_expanded_region {
        return;
    }

    spawn_expanded_subzone_levels(
        &mut spawn_subzone_level_writer,
        &subzone_level_entities,
        &expanded_region,
    );
    despawn_expanded_subzone_levels(
        &mut collaspe_zone_level_writer,
        &subzone_levels,
        &expanded_region,
    );

    *previous_camera_global_transform = global_transform;
    *previous_expanded_region = expanded_region;

    log_if_slow("spawn_subzones_for_camera", start);
}

fn zones_in_sight_distance(focus_pos: Pos) -> Region {
    let from = Zone::from(
        focus_pos
            .offset(PosOffset {
                x: -MAX_VISIBLE_DISTANCE,
                level: LevelOffset::ZERO,
                z: -MAX_VISIBLE_DISTANCE,
            })
            .unwrap(),
    );
    let to = Zone::from(
        focus_pos
            .offset(PosOffset {
                x: MAX_VISIBLE_DISTANCE,
                level: LevelOffset::ZERO,
                z: MAX_VISIBLE_DISTANCE,
            })
            .unwrap(),
    );
    Region::from(&ZoneRegion::new(from.x..=to.x, from.z..=to.z))
}

/** Upper limit for expanding subzones */
fn maximal_expanded_zones(player_zone: Zone) -> Region {
    let x_from = player_zone.x - MAX_EXPAND_DISTANCE;
    let x_to = player_zone.x + MAX_EXPAND_DISTANCE;
    let z_from = player_zone.z - MAX_EXPAND_DISTANCE;
    let z_to = player_zone.z + MAX_EXPAND_DISTANCE;

    Region::from(&ZoneRegion::new(x_from..=x_to, z_from..=z_to))
}

/** Region of expanded zones */
fn expanded_region(focus: &Focus, camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let minimal_expanded_zones = zones_in_sight_distance(Pos::from(focus));
    let maximal_expanded_zones = maximal_expanded_zones(Zone::from(Pos::from(focus)));

    visible_region(camera, global_transform).clamp(&minimal_expanded_zones, &maximal_expanded_zones)
}

/** Region visible on the camera, for both expanded subzones and collapsed zones */
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
    collaspe_zone_level_writer: &mut EventWriter<CollapseZoneLevel>,
    subzone_levels: &Query<&SubzoneLevel>,
    expanded_region: &Region,
) {
    // we use hashmap keys to get rid of duplicates
    let zone_levels = subzone_levels
        .iter()
        .map(|subzone_level| (ZoneLevel::from(*subzone_level), ()))
        .collect::<bevy::utils::HashMap<ZoneLevel, ()>>();

    zone_levels
        .keys()
        .copied()
        .filter(|zone_level| !expanded_region.contains_zone_level(*zone_level))
        .for_each(|zone_level| {
            collaspe_zone_level_writer.send(CollapseZoneLevel { zone_level });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_subzone_levels(
    mut spawn_subzone_level_reader: EventReader<SpawnSubzoneLevel>,
    map_assets: Res<Assets<Map>>,
    mut zone_spawner: ZoneSpawner,
) {
    let start = Instant::now();

    println!(
        "Spawning {} subzone levels",
        spawn_subzone_level_reader.len()
    );

    for spawn_event in &mut spawn_subzone_level_reader {
        zone_spawner.spawn_expanded_subzone_level(&map_assets, spawn_event.subzone_level);
    }

    log_if_slow("spawn_subzone_levels", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn collapse_zone_levels(
    mut commands: Commands,
    mut collapse_zone_level_reader: EventReader<CollapseZoneLevel>,
    mut zone_spawner: ZoneSpawner,
) {
    let start = Instant::now();

    println!(
        "Collapsing {} zone levels",
        collapse_zone_level_reader.len()
    );

    for collapse_event in &mut collapse_zone_level_reader {
        for subzone_level in collapse_event.zone_level.subzone_levels() {
            if let Some(entity) = zone_spawner.subzone_level_entities.get(subzone_level) {
                commands.entity(entity).despawn_recursive();
                zone_spawner.subzone_level_entities.remove(entity);
            }
        }

        match zone_spawner.spawner.explored.has_zone_level_been_seen(
            &zone_spawner.asset_server,
            &zone_spawner.overmap_buffer_assets,
            &mut zone_spawner.overmap_buffer_manager,
            collapse_event.zone_level,
        ) {
            Some(SeenFrom::CloseBy | SeenFrom::FarAway) => {
                let visible = Visibility::Inherited;
                if let Some(zone_level_entity) = zone_spawner
                    .zone_level_entities
                    .get(collapse_event.zone_level)
                {
                    commands.entity(zone_level_entity).insert(visible);
                } else if collapse_event.zone_level.level <= Level::ZERO {
                    zone_spawner.spawn_collapsed_zone_level(collapse_event.zone_level, &visible);
                }
            }
            None | Some(SeenFrom::Never) => {}
        }
    }

    log_if_slow("collapse_zone_levels", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_collapsed_zone_levels(
    mut spawn_zone_level_writer: EventWriter<SpawnZoneLevel>,
    mut update_zone_level_visibility_writer: EventWriter<UpdateZoneLevelVisibility>,
    zone_level_entities: Res<ZoneLevelEntities>,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_visible_region: Local<Region>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    collapsed_zone_levels: Query<(&ZoneLevel, &Children), (With<Collapsed>, With<Visibility>)>,
    new_subzone_levels: Query<(), Added<SubzoneLevel>>,
) {
    // Collapsed zone level visibility: not SeenFrom::Never and not open sky, deep rock, etc.
    // Collapsed zone level child visibility: not expanded, even when zoomed out

    let start = Instant::now();

    /*println!(
        "update_collapsed_zone_levels {:?} {:?}",
        new_subzone_levels.iter().collect::<Vec<_>>().len(),
        new_subzone_levels.is_empty()
    );*/

    let (camera, &global_transform) = cameras.single();
    if global_transform == *previous_camera_global_transform && new_subzone_levels.is_empty() {
        return;
    }

    // Collapsed zones above zero add little value, so we always skip these.
    let visible_region = visible_region(camera, &global_transform).ground_only();
    //println!("Visible region: {:?}", &visible_region);
    if visible_region == *previous_visible_region && new_subzone_levels.is_empty() {
        return;
    }
    //println!("update_collapsed_zone_levels refresh");
    //dbg!(&visible_region);

    for zone_level in visible_region
        .zone_levels()
        .into_iter()
        .map(ZoneLevel::from)
        .collect::<HashSet<_>>()
    {
        if zone_level_entities.get(zone_level).is_none() {
            spawn_zone_level_writer.send(SpawnZoneLevel { zone_level });
        }
    }

    let recalculated_region = visible_region.clamp(&previous_visible_region, &Region::default());
    //println!("Recalculated region: {:?}", &recalculated_region);

    for (&zone_level, children) in collapsed_zone_levels.iter() {
        if recalculated_region.contains_zone_level(zone_level) {
            update_zone_level_visibility_writer.send(UpdateZoneLevelVisibility {
                zone_level,
                children: children.iter().copied().collect(),
            });
        }
    }

    *previous_camera_global_transform = global_transform;
    *previous_visible_region = visible_region;

    log_if_slow("update_collapsed_zone_levels", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_zone_levels(
    mut spawn_zone_level_reader: EventReader<SpawnZoneLevel>,
    player_action_state: Res<PlayerActionState>,
    mut zone_spawner: ZoneSpawner,
    players: Query<&Pos, With<Player>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let start = Instant::now();

    println!("Spawning {} zone levels", spawn_zone_level_reader.len());

    let (camera, &global_transform) = cameras.single();
    let visible_region = visible_region(camera, &global_transform).ground_only();
    let &player_pos = players.single();
    let focus = Focus::new(&player_action_state, player_pos);
    let sight_region = zones_in_sight_distance(Pos::from(&focus));

    for spawn_event in &mut spawn_zone_level_reader {
        let visibility = collapsed_visibility(
            &mut zone_spawner,
            spawn_event.zone_level,
            &sight_region,
            &visible_region,
            &focus,
        );
        zone_spawner.spawn_collapsed_zone_level(spawn_event.zone_level, &visibility);
    }

    log_if_slow("spawn_zone_levels", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_zone_level_visibility(
    mut commands: Commands,
    mut update_zone_level_visibility_reader: EventReader<UpdateZoneLevelVisibility>,
    player_action_state: Res<PlayerActionState>,
    mut zone_spawner: ZoneSpawner,
    players: Query<&Pos, With<Player>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let start = Instant::now();

    println!(
        "Updating the visibility of {} zone levels",
        update_zone_level_visibility_reader.len()
    );

    let (camera, &global_transform) = cameras.single();
    let visible_region = visible_region(camera, &global_transform).ground_only();
    let &player_pos = players.single();
    let focus = Focus::new(&player_action_state, player_pos);
    let sight_region = zones_in_sight_distance(Pos::from(&focus));

    for update_zone_level_visibility_event in &mut update_zone_level_visibility_reader {
        let visibility = collapsed_visibility(
            &mut zone_spawner,
            update_zone_level_visibility_event.zone_level,
            &sight_region,
            &visible_region,
            &focus,
        );
        //println!("{collapsed_zone_level:?} becomes {visibility:?}");
        for &entity in &update_zone_level_visibility_event.children {
            // Removing 'Visibility' and 'ComputedVisibility' is not more performant in Bevy 0.9
            commands.entity(entity).insert(visibility);
        }
    }

    log_if_slow("update_zone_level_visibility", start);
}

fn collapsed_visibility(
    zone_spawner: &mut ZoneSpawner,
    zone_level: ZoneLevel,
    sight_region: &Region,
    visible_region: &Region,
    focus: &Focus,
) -> Visibility {
    if zone_level.level == Level::from(focus).min(Level::ZERO)
        && zone_level.subzone_levels().iter().all(|subzone_level| {
            visible_region.contains_zone_level(ZoneLevel::from(*subzone_level))
                && !sight_region.contains_zone_level(ZoneLevel::from(*subzone_level))
                && zone_spawner.spawner.explored.has_zone_level_been_seen(
                    &zone_spawner.asset_server,
                    &zone_spawner.overmap_buffer_assets,
                    &mut zone_spawner.overmap_buffer_manager,
                    zone_level,
                ) == Some(SeenFrom::FarAway)
        })
    {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_map_events(
    mut zone_spawner: ZoneSpawner,
    mut map_asset_events: EventReader<AssetEvent<Map>>,
    map_assets: Res<Assets<Map>>,
) {
    for map_asset_event in &mut map_asset_events {
        if let AssetEvent::Created { handle } = map_asset_event {
            let map = map_assets.get(handle).expect("Map loaded");
            for submap in &map.0 {
                let subzone_level = SubzoneLevel {
                    x: submap.coordinates.0,
                    level: Level::new(submap.coordinates.2),
                    z: submap.coordinates.1,
                };
                if zone_spawner
                    .subzone_level_entities
                    .get(subzone_level)
                    .is_none()
                {
                    assert_eq!(
                        submap.coordinates,
                        subzone_level.coordinates(),
                        "The stored coordinates and calculated coordinated should match"
                    );
                    zone_spawner.spawn_subzone(submap, subzone_level);
                }
            }
            zone_spawner.map_manager.mark_loaded(handle);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_overmap_buffer_events(
    mut overmap_buffer_asset_events: EventReader<AssetEvent<OvermapBuffer>>,
    overmap_buffer_assets: Res<Assets<OvermapBuffer>>,
    mut overmap_buffer_manager: ResMut<OvermapBufferManager>,
    mut explored: ResMut<Explored>,
) {
    let start = Instant::now();

    for overmap_buffer_asset_event in &mut overmap_buffer_asset_events {
        if let AssetEvent::Created { handle } = overmap_buffer_asset_event {
            let overzone = overmap_buffer_manager.mark_loaded(handle);
            let overmap_buffer = overmap_buffer_assets
                .get(handle)
                .expect("Overmap buffer loaded");
            explored.load(overzone, overmap_buffer);
        }
    }

    log_if_slow("update_zone_level_visibility", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_overmap_events(
    mut overmap_asset_events: EventReader<AssetEvent<Overmap>>,
    overmap_assets: Res<Assets<Overmap>>,
    mut overmap_manager: ResMut<OvermapManager>,
    mut zone_level_ids: ResMut<ZoneLevelIds>,
) {
    let start = Instant::now();

    for overmap_asset_event in &mut overmap_asset_events {
        if let AssetEvent::Created { handle } = overmap_asset_event {
            let overzone = overmap_manager.mark_loaded(handle);
            let overmap = overmap_assets.get(handle).expect("Overmap loaded");
            zone_level_ids.load(overzone, overmap);
        }
    }

    log_if_slow("update_zone_level_visibility", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_zone_levels_with_missing_assets(
    player_action_state: Res<PlayerActionState>,
    mut zone_spawner: ZoneSpawner,
    zone_levels: Query<(Entity, &ZoneLevel), With<MissingAsset>>,
    players: Query<&Pos, With<Player>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let start = Instant::now();

    if zone_levels.iter().len() == 0 {
        return;
    }

    let (camera, &global_transform) = cameras.single();
    let &player_pos = players.single();
    let visible_region = visible_region(camera, &global_transform).ground_only();
    let focus = Focus::new(&player_action_state, player_pos);
    let sight_region = zones_in_sight_distance(Pos::from(&focus));

    for (entity, &zone_level) in &zone_levels {
        let Some(seen_from) = zone_spawner.spawner.explored.has_zone_level_been_seen(
            &zone_spawner.asset_server,
            &zone_spawner.overmap_buffer_assets,
            &mut zone_spawner.overmap_buffer_manager,
            zone_level,
        ) else {
            continue;
        };

        let Some(definition) = zone_spawner
            .zone_level_ids
            .get(
                &zone_spawner.asset_server,
                &zone_spawner.overmap_assets,
                &mut zone_spawner.overmap_manager,
                zone_level,
            )
            .map(|object_id| ObjectDefinition {
                category: ObjectCategory::ZoneLevel,
                id: object_id.clone(),
            })
        else {
            continue;
        };

        let child_visibility = collapsed_visibility(
            &mut zone_spawner,
            zone_level,
            &sight_region,
            &visible_region,
            &focus,
        );

        zone_spawner.complete_collapsed_zone_level(
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
