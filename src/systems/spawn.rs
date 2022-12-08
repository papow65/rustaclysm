use super::log_if_slow;
use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use std::time::Instant;

const MAX_EXPAND_DISTANCE: i32 = 10;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_nearby_overzones(
    mut tile_spawner: TileSpawner,
    all_zone_levels: Query<(Entity, &ZoneLevel, Option<&Collapsed>)>,
    players: Query<(&Pos, &Player)>,
) {
    let start = Instant::now();

    // TODO more than once
    if all_zone_levels.is_empty() {
        if let Ok((&player_pos, player)) = players.get_single() {
            let focus = Focus::new(player, player_pos);
            tile_spawner.spawn_zones_around(Zone::from(ZoneLevel::from(&focus)));
        }
    }

    log_if_slow("spawn_nearby_overzones", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_zones_for_camera(
    mut commands: Commands,
    mut tile_spawner: TileSpawner,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    players: Query<(&Pos, &Player)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    expanded_zone_levels: Query<(Entity, &ZoneLevel), Without<Collapsed>>,
) {
    let start = Instant::now();

    let (camera, &global_transform) = cameras.single();

    if global_transform == *previous_camera_global_transform {
        return;
    }

    let (&player_pos, player) = players.single();
    let focus = Focus::new(player, player_pos);
    let expanded_region = expanded_region(&focus, camera, &global_transform);

    spawn_expanded_zone_levels(&mut tile_spawner, &expanded_zone_levels, &expanded_region);
    despawn_expanded_zone_levels(&mut commands, &expanded_zone_levels, &expanded_region);

    *previous_camera_global_transform = global_transform;

    log_if_slow("spawn_zones_for_camera", start);
}

fn minimal_expanded_zones(player_pos: Pos) -> ZoneRegion {
    ZoneRegion::new(
        Zone::from(ZoneLevel::from(
            player_pos
                .offset(Pos::new(
                    -MAX_VISIBLE_DISTANCE,
                    Level::ZERO,
                    -MAX_VISIBLE_DISTANCE,
                ))
                .unwrap(),
        )),
        Zone::from(ZoneLevel::from(
            player_pos
                .offset(Pos::new(
                    MAX_VISIBLE_DISTANCE,
                    Level::ZERO,
                    MAX_VISIBLE_DISTANCE,
                ))
                .unwrap(),
        )),
    )
}

fn maximal_expanded_zones(player_zone: Zone) -> ZoneRegion {
    ZoneRegion::new(
        Zone {
            x: player_zone.x - MAX_EXPAND_DISTANCE,
            z: player_zone.z - MAX_EXPAND_DISTANCE,
        },
        Zone {
            x: player_zone.x + MAX_EXPAND_DISTANCE,
            z: player_zone.z + MAX_EXPAND_DISTANCE,
        },
    )
}

fn expanded_region(focus: &Focus, camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let minimal_expanded_zones = minimal_expanded_zones(Pos::from(focus));
    let maximal_expanded_zones = maximal_expanded_zones(Zone::from(ZoneLevel::from(focus)));

    assert!(minimal_expanded_zones.well_formed());
    assert!(maximal_expanded_zones.well_formed());
    assert!(maximal_expanded_zones.contains_zone_region(&minimal_expanded_zones));

    visible_region(focus, camera, global_transform).clamp(
        &Region::from(&minimal_expanded_zones),
        &Region::from(&maximal_expanded_zones),
    )
}

fn visible_region(focus: &Focus, camera: &Camera, global_transform: &GlobalTransform) -> Region {
    let zone_levels = visible_area(camera, global_transform)
        .into_iter()
        .filter(|zone_level| focus.is_shown(zone_level.level))
        .collect::<Vec<ZoneLevel>>();
    Region::new(&zone_levels)
}

fn visible_area(camera: &Camera, global_transform: &GlobalTransform) -> Vec<ZoneLevel> {
    let Some((corner_a, corner_b)) = camera.logical_viewport_rect() else {
        return Vec::new();
    };

    let mut zone_levels = Vec::new();
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
                zone_levels.push(ZoneLevel::from(Pos {
                    x: ground_x.floor() as i32,
                    level,
                    z: ground_z.floor() as i32,
                }));
                zone_levels.push(ZoneLevel::from(Pos {
                    x: ground_x.ceil() as i32,
                    level,
                    z: ground_z.ceil() as i32,
                }));
            }
        }
    }

    zone_levels
}

fn spawn_expanded_zone_levels(
    tile_spawner: &mut TileSpawner,
    expanded_zone_levels: &Query<(Entity, &ZoneLevel), Without<Collapsed>>,
    expanded_region: &Region,
) {
    let expanded_zone_levels = expanded_zone_levels
        .iter()
        .map(|(_, &zone_level)| zone_level)
        .collect::<HashSet<_>>();

    for zone_level in expanded_region.zone_levels() {
        if !expanded_zone_levels.contains(&zone_level) {
            if let Err(e) = tile_spawner.spawn_expanded_zone_level(zone_level) {
                //eprintln!("While loading {zone_level:?}: {e}");
                panic!("While loading {zone_level:?}: {e}");
            }
        }
    }
}

fn despawn_expanded_zone_levels(
    commands: &mut Commands,
    expanded_zone_levels: &Query<(Entity, &ZoneLevel), Without<Collapsed>>,
    expanded_region: &Region,
) {
    expanded_zone_levels
        .iter()
        .filter(|(_, &expanded_zone_level)| {
            !expanded_region.contains_zone_level(expanded_zone_level)
        })
        .for_each(|(e, &_expanded_zone_level)| {
            commands.entity(e).despawn_recursive();
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_collapsed_zone_levels(
    mut commands: Commands,
    mut explored: ResMut<Explored>,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut previous_visible_region: Local<Region>,
    players: Query<(&Pos, &Player)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    collapsed_zone_levels: Query<(&ZoneLevel, &Children), (With<Collapsed>, With<Visibility>)>,
) {
    // Collapsed zone level visibility: not SeenFrom::Never and not open sky, deep rock, etc.
    // Collapsed zone level child visibility: not expanded, even when zoomed out

    let start = Instant::now();

    let (camera, &global_transform) = cameras.single();
    if global_transform == *previous_camera_global_transform {
        return;
    }

    let (&player_pos, player) = players.single();
    let focus = Focus::new(player, player_pos);

    let expanded_region = expanded_region(&focus, camera, &global_transform);
    //println!("Expanded region: {:?}", &expanded_region);
    let visible_region = visible_region(&focus, camera, &global_transform);
    //println!("Visible region: {:?}", &visible_region);
    let recalculated_region = visible_region.clamp(&previous_visible_region, &Region::default());
    //println!("Recalculated region: {:?}", &recalculated_region);

    for (&collapsed_zone_level, children) in collapsed_zone_levels.iter() {
        if recalculated_region.contains_zone_level(collapsed_zone_level) {
            //println!("{collapsed_zone_level:?} visibility?");
            let visibility = Visibility {
                is_visible: if expanded_region.contains_zone_level(collapsed_zone_level) {
                    explored.has_zone_level_been_seen(collapsed_zone_level) == SeenFrom::FarAway
                } else {
                    visible_region.contains_zone_level(collapsed_zone_level)
                },
            };

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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_changed_markers(
    mut commands: Commands,
    level_changers: Query<Entity, With<LevelChanged>>,
) {
    let start = Instant::now();

    for entity in level_changers.iter() {
        commands.entity(entity).remove::<LevelChanged>();
    }

    log_if_slow("remove_changed_markers", start);
}
