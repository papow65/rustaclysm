use super::log_if_slow;
use crate::prelude::*;
use bevy::prelude::*;
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

    let (camera, global_transform) = cameras.single();

    if *global_transform == *previous_camera_global_transform {
        return;
    }

    let (&player_pos, player) = players.single();
    let focus = Focus::new(player, player_pos);
    let (minimal_start, minimal_end) = minimal_expanded_zones(Pos::from(&focus));
    let (maximal_start, maximal_end) = maximal_expanded_zones(Zone::from(ZoneLevel::from(&focus)));
    let expanded_zones = expanded_zones(
        &focus,
        camera,
        global_transform,
        maximal_start,
        minimal_start,
        minimal_end,
        maximal_end,
    );
    //println!("Expanded zones: {:?}", expanded_zones);

    spawn_expanded_zone_levels(&mut tile_spawner, &expanded_zone_levels, &expanded_zones);
    despawn_expanded_zone_levels(&mut commands, &expanded_zone_levels, &expanded_zones);

    *previous_camera_global_transform = *global_transform;

    log_if_slow("spawn_zones_for_camera", start);
}

fn minimal_expanded_zones(player_pos: Pos) -> (Zone, Zone) {
    (
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

fn maximal_expanded_zones(player_zone: Zone) -> (Zone, Zone) {
    (
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

fn expanded_zones(
    focus: &Focus,
    camera: &Camera,
    global_transform: &GlobalTransform,
    maximal_start: Zone,
    minimal_start: Zone,
    minimal_end: Zone,
    maximal_end: Zone,
) -> [(Zone, Zone); Level::AMOUNT] {
    assert!(maximal_start.x <= minimal_start.x);
    assert!(minimal_start.x <= minimal_end.x);
    assert!(minimal_end.x <= maximal_end.x);

    assert!(maximal_start.z <= minimal_start.z);
    assert!(minimal_start.z <= minimal_end.z);
    assert!(minimal_end.z <= maximal_end.z);

    let mut i = 0;
    visible_area(camera, global_transform).map(|(visible_xs, visible_zs)| {
        let level = Level::ALL[i];
        i += 1;

        //println!("Visible: x {:?}, z {:?}", &visible_xs, visible_zs);
        if !focus.is_shown(level) || visible_xs.len() < 2 || visible_zs.len() < 2 {
            return (minimal_start, minimal_end);
        }

        let min_x = visible_xs.iter().copied().min().unwrap();
        let min_z = visible_zs.iter().copied().min().unwrap();
        let mut start = Zone::from(ZoneLevel::from(Pos::new(min_x, Level::ZERO, min_z)));
        start.x = start.x.clamp(maximal_start.x, minimal_start.x);
        start.z = start.z.clamp(maximal_start.z, minimal_start.z);

        let max_x = visible_xs.iter().copied().max().unwrap();
        let max_z = visible_zs.iter().copied().max().unwrap();
        let mut end = Zone::from(ZoneLevel::from(Pos::new(max_x, Level::ZERO, max_z)));
        end.x = end.x.clamp(minimal_end.x, maximal_end.x);
        end.z = end.z.clamp(minimal_end.z, maximal_end.z);

        (start, end)
    })
}

fn visible_area(
    camera: &Camera,
    global_transform: &GlobalTransform,
) -> [(Vec<i32>, Vec<i32>); Level::AMOUNT] {
    let Some((corner_a, corner_b)) = camera.logical_viewport_rect() else {
        return [(); Level::AMOUNT].map(|_| (Vec::new(), Vec::new()));
    };

    let mut i = 0;
    [(); Level::AMOUNT]
        .map(|_| {
            let level = Level::ALL[i];
            i += 1;
            level
        })
        .map(|level| {
            let mut xs = Vec::new();
            let mut zs = Vec::new();
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
                    {
                        let ground_x = origin.x + k * direction.x;
                        xs.push(ground_x.floor() as i32);
                        xs.push(ground_x.ceil() as i32);
                    }
                    {
                        let ground_z = origin.z + k * direction.z;
                        zs.push(ground_z.floor() as i32);
                        zs.push(ground_z.ceil() as i32);
                    }
                }
            }
            (xs, zs)
        })
}

fn spawn_expanded_zone_levels(
    tile_spawner: &mut TileSpawner,
    expanded_zone_levels: &Query<(Entity, &ZoneLevel), Without<Collapsed>>,
    expanded_zones: &[(Zone, Zone); Level::AMOUNT],
) {
    for y in Level::ALL {
        let (expanded_start, expanded_end) = expanded_zones[y.index()];
        for x in expanded_start.x..=expanded_end.x {
            for z in expanded_start.z..=expanded_end.z {
                let zone_level = Zone { x, z }.zone_level(y);
                let zone_exists = expanded_zone_levels
                    .iter()
                    .any(|(_, &expanded_zone_level)| expanded_zone_level == zone_level);
                if !zone_exists {
                    if let Err(e) = tile_spawner.spawn_expanded_zone_level(zone_level) {
                        eprintln!("{e}");
                        panic!("{e}");
                    }
                }
            }
        }
    }
}

fn despawn_expanded_zone_levels(
    commands: &mut Commands,
    expanded_zone_levels: &Query<(Entity, &ZoneLevel), Without<Collapsed>>,
    expanded_zones: &[(Zone, Zone); Level::AMOUNT],
) {
    expanded_zone_levels
        .iter()
        .filter(|(_, &expanded_zone_level)| {
            let (expanded_start, expanded_end) = expanded_zones[expanded_zone_level.level.index()];
            !Zone::from(expanded_zone_level).in_range(expanded_start, expanded_end)
        })
        .for_each(|(e, &_expanded_zone_level)| {
            commands.entity(e).despawn_recursive();
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_collapsed_zone_levels(
    mut commands: Commands,
    mut explored: ResMut<Explored>,
    mut previous_player_pos: Local<Pos>,
    players: Query<&Pos, With<Player>>,
    collapsed_zone_levels: Query<(&ZoneLevel, &Children), (With<Collapsed>, With<Visibility>)>,
) {
    // Collapsed zone level visibility: not SeenFrom::Never
    // Collapsed zone level child visibility: not expanded, even when zoomed out

    let start = Instant::now();

    let Ok(&player_pos) = players.get_single() else {
        return;
    };
    if player_pos == *previous_player_pos {
        return;
    }

    let (maximal_start, maximal_end) =
        maximal_expanded_zones(Zone::from(ZoneLevel::from(player_pos)));
    println!(
        "update_collapsed_zone_levels: {:?} {:?}",
        &maximal_start, &maximal_end
    );

    for (&collapsed_zone_level, children) in collapsed_zone_levels.iter() {
        //println!("{collapsed_zone_level:?} visibility?");
        let visibility = if Zone::from(collapsed_zone_level).in_range(maximal_start, maximal_end) {
            if explored.has_zone_level_been_seen(collapsed_zone_level) == SeenFrom::FarAway {
                Visibility::VISIBLE
            } else {
                Visibility::INVISIBLE
            }
        } else {
            Visibility::VISIBLE
        };

        for &entity in children.iter() {
            //println!("{collapsed_zone_level:?} becomes {visibility:?}");

            // Removing 'Visibility' and 'ComputedVisibility' is not more performant in Bevy 0.9
            commands.entity(entity).insert(visibility.clone());
        }
    }

    *previous_player_pos = player_pos;

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
