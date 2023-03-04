use super::log_if_slow;
use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use std::time::Instant;

const MAX_EXPAND_DISTANCE: i32 = 20;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_zones_for_camera(
    mut commands: Commands,
    mut tile_spawner: TileSpawner,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_expanded_region: Local<Region>,
    players: Query<(&Pos, &Player)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    expanded_subzone_levels: Query<(Entity, &SubzoneLevel), Without<Collapsed>>,
) {
    let start = Instant::now();

    let (camera, &global_transform) = cameras.single();
    if global_transform == *previous_camera_global_transform {
        return;
    }

    let (&player_pos, player) = players.single();
    let focus = Focus::new(player, player_pos);
    let expanded_region = expanded_region(&focus, camera, &global_transform);
    if expanded_region == *previous_expanded_region {
        return;
    }

    spawn_expanded_subzone_levels(&mut tile_spawner, &expanded_region);
    despawn_expanded_subzone_levels(
        &mut commands,
        &mut tile_spawner,
        &expanded_subzone_levels,
        &expanded_region,
    );

    *previous_camera_global_transform = global_transform;
    *previous_expanded_region = expanded_region;

    log_if_slow("spawn_zones_for_camera", start);
}

fn minimal_expanded_zones(player_pos: Pos) -> SubzoneRegion {
    let from = SubzoneLevel::from(
        player_pos
            .offset(PosOffset {
                x: -MAX_VISIBLE_DISTANCE,
                level: LevelOffset::ZERO,
                z: -MAX_VISIBLE_DISTANCE,
            })
            .unwrap(),
    );
    let to = SubzoneLevel::from(
        player_pos
            .offset(PosOffset {
                x: MAX_VISIBLE_DISTANCE,
                level: LevelOffset::ZERO,
                z: MAX_VISIBLE_DISTANCE,
            })
            .unwrap(),
    );
    SubzoneRegion::new(from.x..=to.x, from.z..=to.z)
}

fn maximal_expanded_zones(player_subzone_level: SubzoneLevel) -> SubzoneRegion {
    let x_from = player_subzone_level.x - MAX_EXPAND_DISTANCE;
    let x_to = player_subzone_level.x + MAX_EXPAND_DISTANCE;
    let z_from = player_subzone_level.z - MAX_EXPAND_DISTANCE;
    let z_to = player_subzone_level.z + MAX_EXPAND_DISTANCE;

    SubzoneRegion::new(x_from..=x_to, z_from..=z_to)
}

fn expanded_region(focus: &Focus, camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let minimal_expanded_zones = minimal_expanded_zones(Pos::from(focus));
    let maximal_expanded_zones = maximal_expanded_zones(SubzoneLevel::from(Pos::from(focus)));

    visible_region(focus, camera, global_transform).clamp(
        &Region::from(&minimal_expanded_zones),
        &Region::from(&maximal_expanded_zones),
    )
}

fn visible_region(focus: &Focus, camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let zone_levels = visible_area(camera, global_transform)
        .into_iter()
        .filter(|zone_level| focus.is_zone_level_shown(zone_level.level))
        .collect::<Vec<SubzoneLevel>>();
    Region::new(&zone_levels)
}

fn visible_area(camera: &Camera, global_transform: &GlobalTransform) -> Vec<SubzoneLevel> {
    let Some((corner_a, corner_b)) = camera.logical_viewport_rect() else {
        return Vec::new();
    };

    let mut subzone_levels = Vec::new();
    for level in Level::ALL {
        for corner in [
            Vec2::new(corner_a.x, corner_a.y),
            Vec2::new(corner_a.x, corner_b.y),
            Vec2::new(corner_b.x, corner_a.y),
            Vec2::new(corner_b.x, corner_b.y),
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

    subzone_levels
}

fn spawn_expanded_subzone_levels(tile_spawner: &mut TileSpawner, expanded_region: &Region) {
    for subzone_level in expanded_region.subzone_levels() {
        let expanded = !tile_spawner
            .zone_level_ids
            .get(ZoneLevel::from(subzone_level))
            .is_hidden_zone();
        if expanded {
            let missing = tile_spawner
                .subzone_level_entities
                .get(subzone_level)
                .is_none();
            if missing {
                if let Err(e) = tile_spawner.spawn_expanded_subzone_level(subzone_level) {
                    panic!(
                        "While loading {subzone_level:?} in {:?}: {e}",
                        ZoneLevel::from(subzone_level)
                    );
                }
            }
        }
    }
}

fn despawn_expanded_subzone_levels(
    commands: &mut Commands,
    tile_spawner: &mut TileSpawner,
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
            tile_spawner.subzone_level_entities.remove(e);

            let zone_level = ZoneLevel::from(expanded_subzone_level);
            if tile_spawner.explored.has_zone_level_been_seen(zone_level) != SeenFrom::Never {
                let visible = Visibility { is_visible: true };
                if let Some(zone_level_entity) = tile_spawner.zone_level_entities.get(zone_level) {
                    commands.entity(zone_level_entity).insert(visible);
                } else {
                    tile_spawner.spawn_collapsed_zone_level(zone_level, &visible);
                }
            }
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_collapsed_zone_levels(
    mut commands: Commands,
    mut tile_spawner: TileSpawner,
    mut skip_once: Local<bool>,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_visible_region: Local<Region>,
    players: Query<(&Pos, &Player)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    collapsed_zone_levels: Query<(&ZoneLevel, &Children), (With<Collapsed>, With<Visibility>)>,
) {
    // Collapsed zone level visibility: not SeenFrom::Never and not open sky, deep rock, etc.
    // Collapsed zone level child visibility: not expanded, even when zoomed out

    let start = Instant::now();

    // We need to wait for a sync, after zones have been expanded.
    if !*skip_once {
        *skip_once = true;
        return;
    }

    let (camera, &global_transform) = cameras.single();
    if global_transform == *previous_camera_global_transform {
        return;
    }

    let (&player_pos, player) = players.single();
    let focus = Focus::new(player, player_pos);
    let visible_region = visible_region(&focus, camera, &global_transform);
    //println!("Visible region: {:?}", &visible_region);
    if visible_region == *previous_visible_region {
        return;
    }
    let expanded_region = expanded_region(&focus, camera, &global_transform);
    //println!("Expanded region: {:?}", &expanded_region);

    for zone_level in visible_region
        .subzone_levels()
        .into_iter()
        .map(ZoneLevel::from)
        .collect::<HashSet<_>>()
    {
        if tile_spawner.zone_level_entities.get(zone_level).is_none() {
            let visibility = collapsed_visibility(
                &mut tile_spawner.explored,
                zone_level,
                &expanded_region,
                &visible_region,
            );
            tile_spawner.spawn_collapsed_zone_level(zone_level, &visibility);
        }
    }

    let recalculated_region = visible_region.clamp(&previous_visible_region, &Region::default());
    //println!("Recalculated region: {:?}", &recalculated_region);

    for (&collapsed_zone_level, children) in collapsed_zone_levels.iter() {
        if recalculated_region
            .contains_subzone_level(SubzoneLevel::from(collapsed_zone_level.base_pos()))
        {
            //println!("{collapsed_zone_level:?} visibility?");
            let visibility = collapsed_visibility(
                &mut tile_spawner.explored,
                collapsed_zone_level,
                &expanded_region,
                &visible_region,
            );
            for &entity in children.iter() {
                //println!("{collapsed_zone_level:?} becomes {visibility:?}");

                // Removing 'Visibility' and 'ComputedVisibility' is not more performant in Bevy 0.9
                commands.entity(entity).insert(visibility.clone());
            }
        }
    }

    *previous_camera_global_transform = global_transform;
    *previous_visible_region = visible_region;

    log_if_slow("update_collapsed_zone_levels", start);
}

fn collapsed_visibility(
    explored: &mut Explored,
    zone_level: ZoneLevel,
    expanded_region: &Region,
    visible_region: &Region,
) -> Visibility {
    Visibility {
        is_visible: zone_level.subzone_levels().iter().all(|subzone_level| {
            if expanded_region.contains_subzone_level(*subzone_level) {
                explored.has_zone_level_been_seen(zone_level) == SeenFrom::FarAway
            } else {
                visible_region.contains_subzone_level(*subzone_level)
            }
        }),
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn toggle_doors(
    mut commands: Commands,
    mut tile_spawner: TileSpawner,
    toggled: Query<
        (
            Entity,
            &ObjectDefinition,
            &Pos,
            Option<&Openable>,
            Option<&Closeable>,
            &Parent,
        ),
        With<Toggle>,
    >,
) {
    let start = Instant::now();

    for (entity, definition, &pos, openable, closeable, parent) in toggled.iter() {
        assert_ne!(openable.is_some(), closeable.is_some());
        commands.entity(entity).despawn_recursive();
        let TerrainInfo::Terrain{close, open, ..} = tile_spawner.infos.terrain(&definition.id).unwrap() else {panic!()};
        let toggled_id = openable.map_or(close, |_| open).as_ref().unwrap().clone();
        tile_spawner.spawn_terrain(parent.get(), pos, toggled_id);
        commands.spawn(RefreshVisualizations);
    }

    log_if_slow("toggle_doors", start);
}
