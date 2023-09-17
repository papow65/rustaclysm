use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use std::time::Instant;

const MAX_EXPAND_DISTANCE: i32 = 20;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_subzones_for_camera(
    mut commands: Commands,
    player_action_state: Res<PlayerActionState>,
    mut zone_spawner: ZoneSpawner,
    map_assets: Res<Assets<Map>>,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_expanded_region: Local<Region>,
    players: Query<&Pos, With<Player>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    expanded_subzone_levels: Query<(Entity, &SubzoneLevel), Without<Collapsed>>,
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

    spawn_expanded_subzone_levels(&mut zone_spawner, &map_assets, &expanded_region);
    despawn_expanded_subzone_levels(
        &mut commands,
        &mut zone_spawner,
        &expanded_subzone_levels,
        &expanded_region,
    );

    *previous_camera_global_transform = global_transform;
    *previous_expanded_region = expanded_region;

    log_if_slow("spawn_subzones_for_camera", start);
}

fn subzones_in_sight_distance(focus_pos: Pos) -> Region {
    let from = SubzoneLevel::from(
        focus_pos
            .offset(PosOffset {
                x: -MAX_VISIBLE_DISTANCE,
                level: LevelOffset::ZERO,
                z: -MAX_VISIBLE_DISTANCE,
            })
            .unwrap(),
    );
    let to = SubzoneLevel::from(
        focus_pos
            .offset(PosOffset {
                x: MAX_VISIBLE_DISTANCE,
                level: LevelOffset::ZERO,
                z: MAX_VISIBLE_DISTANCE,
            })
            .unwrap(),
    );
    Region::from(&SubzoneRegion::new(from.x..=to.x, from.z..=to.z))
}

/** Upper limit for expanding subzones */
fn maximal_expanded_subzones(player_subzone_level: SubzoneLevel) -> Region {
    let x_from = player_subzone_level.x - MAX_EXPAND_DISTANCE;
    let x_to = player_subzone_level.x + MAX_EXPAND_DISTANCE;
    let z_from = player_subzone_level.z - MAX_EXPAND_DISTANCE;
    let z_to = player_subzone_level.z + MAX_EXPAND_DISTANCE;

    Region::from(&SubzoneRegion::new(x_from..=x_to, z_from..=z_to))
}

/** Region of expanded subzones */
fn expanded_region(focus: &Focus, camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let minimal_expanded_subzones = subzones_in_sight_distance(Pos::from(focus));
    let maximal_expanded_subzones = maximal_expanded_subzones(SubzoneLevel::from(Pos::from(focus)));

    visible_region(camera, global_transform)
        .clamp(&minimal_expanded_subzones, &maximal_expanded_subzones)
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

    let mut subzone_levels = Vec::new();
    for level in Level::ALL {
        for corner in [
            Vec2::new(corner_min.x, corner_min.y),
            Vec2::new(corner_min.x, corner_max.y),
            Vec2::new(corner_max.x, corner_min.y),
            Vec2::new(corner_max.x, corner_max.y),
        ] {
            let Some(Ray { origin, direction }) =
                camera.viewport_to_world(global_transform, corner)
            else {
                continue;
            };

            let k = (level.f32() - origin.y) / direction.y;
            // The camera only looks forward.
            if 0.0 < k {
                let ground_x = origin.x + k * direction.x;
                let ground_z = origin.z + k * direction.z;
                subzone_levels.push(SubzoneLevel::from(Pos {
                    x: ground_x.floor() as i32,
                    level,
                    z: ground_z.floor() as i32,
                }));
                subzone_levels.push(SubzoneLevel::from(Pos {
                    x: ground_x.ceil() as i32,
                    level,
                    z: ground_z.ceil() as i32,
                }));
            }
        }
    }

    Region::new(&subzone_levels)
}

fn spawn_expanded_subzone_levels(
    zone_spawner: &mut ZoneSpawner,
    map_assets: &Assets<Map>,
    expanded_region: &Region,
) {
    for subzone_level in expanded_region.subzone_levels() {
        let missing = zone_spawner
            .subzone_level_entities
            .get(subzone_level)
            .is_none();
        if missing {
            zone_spawner.spawn_expanded_subzone_level(map_assets, subzone_level);
        }
    }
}

fn despawn_expanded_subzone_levels(
    commands: &mut Commands,
    zone_spawner: &mut ZoneSpawner,
    expanded_zone_levels: &Query<(Entity, &SubzoneLevel), Without<Collapsed>>,
    expanded_region: &Region,
) {
    expanded_zone_levels
        .iter()
        .filter(|(_, &expanded_subzone_level)| {
            !expanded_region.contains_subzone_level(expanded_subzone_level)
        })
        .for_each(|(e, &expanded_subzone_level)| {
            commands.entity(e).despawn_recursive();
            zone_spawner.subzone_level_entities.remove(e);

            let zone_level = ZoneLevel::from(expanded_subzone_level);
            match zone_spawner.spawner.explored.has_zone_level_been_seen(
                &zone_spawner.asset_server,
                &mut zone_spawner.overmap_buffer_manager,
                zone_level,
            ) {
                Some(SeenFrom::CloseBy | SeenFrom::FarAway) => {
                    let visible = Visibility::Inherited;
                    if let Some(zone_level_entity) =
                        zone_spawner.zone_level_entities.get(zone_level)
                    {
                        commands.entity(zone_level_entity).insert(visible);
                    } else if zone_level.level <= Level::ZERO {
                        zone_spawner
                            .spawn_collapsed_zone_level(zone_level, &visible)
                            .ok();
                    }
                }
                None | Some(SeenFrom::Never) => {}
            }
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_collapsed_zone_levels(
    mut commands: Commands,
    player_action_state: Res<PlayerActionState>,
    mut zone_spawner: ZoneSpawner,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_visible_region: Local<Region>,
    players: Query<&Pos, With<Player>>,
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

    let &player_pos = players.single();
    // Collapsed zones above zero add little value, so we always skip these.
    let visible_region = visible_region(camera, &global_transform).ground_only();
    //println!("Visible region: {:?}", &visible_region);
    if visible_region == *previous_visible_region && new_subzone_levels.is_empty() {
        return;
    }
    //println!("update_collapsed_zone_levels refresh");
    let focus = Focus::new(&player_action_state, player_pos);
    let sight_region = subzones_in_sight_distance(Pos::from(&focus));
    //println!("Sight region: {:?}", &sight_region);

    for zone_level in visible_region
        .subzone_levels()
        .into_iter()
        .map(ZoneLevel::from)
        .collect::<HashSet<_>>()
    {
        if zone_spawner.zone_level_entities.get(zone_level).is_none() {
            let visibility = collapsed_visibility(
                &mut zone_spawner,
                zone_level,
                &sight_region,
                &visible_region,
                &focus,
            );
            zone_spawner
                .spawn_collapsed_zone_level(zone_level, &visibility)
                .ok();
        }
    }

    let recalculated_region = visible_region.clamp(&previous_visible_region, &Region::default());
    //println!("Recalculated region: {:?}", &recalculated_region);

    for (&collapsed_zone_level, children) in collapsed_zone_levels.iter() {
        if recalculated_region
            .contains_subzone_level(SubzoneLevel::from(collapsed_zone_level.base_pos()))
        {
            update_zone_level_visualization(
                &mut commands,
                &mut zone_spawner,
                collapsed_zone_level,
                &sight_region,
                &visible_region,
                &focus,
                children,
            );
        }
    }

    *previous_camera_global_transform = global_transform;
    *previous_visible_region = visible_region;

    log_if_slow("update_collapsed_zone_levels", start);
}

fn update_zone_level_visualization(
    commands: &mut Commands,
    zone_spawner: &mut ZoneSpawner,
    collapsed_zone_level: ZoneLevel,
    sight_region: &Region,
    visible_region: &Region,
    focus: &Focus,
    children: &Children,
) {
    //println!("{collapsed_zone_level:?} visibility?");
    let visibility = collapsed_visibility(
        zone_spawner,
        collapsed_zone_level,
        sight_region,
        visible_region,
        focus,
    );
    //println!("{collapsed_zone_level:?} becomes {visibility:?}");
    for &entity in children {
        // Removing 'Visibility' and 'ComputedVisibility' is not more performant in Bevy 0.9
        commands.entity(entity).insert(visibility);
    }
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
            visible_region.contains_subzone_level(*subzone_level)
                && !sight_region.contains_subzone_level(*subzone_level)
                && zone_spawner.spawner.explored.has_zone_level_been_seen(
                    &zone_spawner.asset_server,
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
            let map = &map_assets.get(handle).expect("Map loaded");
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
            zone_spawner.map_manager.finish_loading(handle);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_overmap_buffer_events(
    mut overmap_buffer_asset_events: EventReader<AssetEvent<OvermapBuffer>>,
    overmap_buffer_assets: Res<Assets<OvermapBuffer>>,
    player_action_state: Res<PlayerActionState>,
    mut zone_spawner: ZoneSpawner,
    players: Query<&Pos, With<Player>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    for overmap_buffer_asset_event in &mut overmap_buffer_asset_events {
        if let AssetEvent::Created { handle } = overmap_buffer_asset_event {
            let overmap_buffer = overmap_buffer_assets.get(handle).expect("Map loaded");
            let overzone = zone_spawner.overmap_buffer_manager.finish_loading(handle);
            zone_spawner.spawner.explored.load(overzone, overmap_buffer);

            let (camera, &global_transform) = cameras.single();
            let &player_pos = players.single();
            let visible_region = visible_region(camera, &global_transform).ground_only();
            let focus = Focus::new(&player_action_state, player_pos);
            let sight_region = subzones_in_sight_distance(Pos::from(&focus));

            let (x_range, z_range) = overzone.xz_ranges();
            for x in x_range {
                for z in z_range.clone() {
                    let zone = Zone { x, z };
                    for level in Level::GROUNDS {
                        let zone_level = ZoneLevel { zone, level };
                        if zone_spawner.zone_level_entities.get(zone_level).is_none() {
                            let visibility = collapsed_visibility(
                                &mut zone_spawner,
                                zone_level,
                                &sight_region,
                                &visible_region,
                                &focus,
                            );
                            let result =
                                zone_spawner.spawn_collapsed_zone_level(zone_level, &visibility);
                            assert!(result.is_ok(), "{zone_level:?}");
                        }
                    }
                }
            }
        }
    }
}
