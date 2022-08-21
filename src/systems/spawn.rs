use crate::prelude::*;
use bevy::prelude::*;

const SPAWN_DISTANCE: u32 = 4;
const DESPAWN_DISTANCE: u32 = SPAWN_DISTANCE + 1;

fn get_center_zones(pos: Pos, player: &Player) -> Vec<Zone> {
    let mut positions = vec![pos];
    if let PlayerActionState::ExaminingPos(camera_pos) = player.state {
        positions.push(camera_pos);
    }
    // Not needed for PlayerActionState::ExaminingZoneLevel
    positions
        .iter()
        .map(|&p| Zone::from(p))
        .collect::<Vec<Zone>>()
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_nearby_overzones(
    mut tile_spawner: TileSpawner,
    all_zone_levels: Query<(Entity, &ZoneLevel, Option<&Collapsed>)>,
    players: Query<&Pos, (With<Player>, With<ZoneChanged>)>,
) {
    // TODO more than once
    if all_zone_levels.is_empty() {
        if let Ok(&pos) = players.get_single() {
            tile_spawner.spawn_zones_around(Zone::from(pos));
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_nearby_zones(
    mut commands: Commands,
    location: Res<Location>,
    mut tile_spawner: TileSpawner,
    collapsed_zone_levels: Query<(&ZoneLevel, &Children), With<Collapsed>>,
    players: Query<(&Pos, &Player), With<ZoneChanged>>,
) {
    if let Ok((&pos, player)) = players.get_single() {
        for center_zone in get_center_zones(pos, player) {
            println!("{:?}", center_zone);
            for nearby_zone in center_zone.nearby(SPAWN_DISTANCE) {
                // level 0 always exists
                if !location.exists(nearby_zone.zone_level(Level::ZERO).base_pos()) {
                    for y in Level::ALL {
                        let zone_level = nearby_zone.zone_level(y);
                        match tile_spawner.spawn_expanded_zone_level(zone_level) {
                            Ok(()) => {
                                set_collapsed_zone_level_visibility(
                                    &mut commands,
                                    &collapsed_zone_levels,
                                    zone_level,
                                    false,
                                );
                            }
                            Err(e) => {
                                eprintln!("{e}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_far_zones(
    mut commands: Commands,
    collapsed_zone_levels: Query<(&ZoneLevel, &Children), With<Collapsed>>,
    expanded_zone_levels: Query<(Entity, &ZoneLevel), Without<Collapsed>>,
    players: Query<(&Pos, &Player), With<ZoneChanged>>,
) {
    if let Ok((&pos, player)) = players.get_single() {
        let centers = get_center_zones(pos, player);
        let is_far_away = |zone: Zone| {
            centers
                .iter()
                .map(|&center| zone.dist(center))
                .all(|dist_from_center| DESPAWN_DISTANCE <= dist_from_center)
        };
        expanded_zone_levels
            .iter()
            .filter(|(_, &checked_zone_level)| is_far_away(Zone::from(checked_zone_level)))
            .for_each(|(e, &checked_zone_level)| {
                commands.entity(e).despawn_recursive();
                set_collapsed_zone_level_visibility(
                    &mut commands,
                    &collapsed_zone_levels,
                    checked_zone_level,
                    true,
                );
            });
    }
}

// TODO remove when no longer glitching
fn set_collapsed_zone_level_visibility(
    _commands: &mut Commands,
    collapsed_zone_levels: &Query<(&ZoneLevel, &Children), With<Collapsed>>,
    expanded_zone_level: ZoneLevel,
    _is_visible: bool,
) {
    if let Some((_, children)) = collapsed_zone_levels
        .iter()
        .find(|(&zone_level, _)| zone_level == expanded_zone_level)
    {
        for &_entity in children.iter() {
            //commands.entity(entity).insert(Visibility { is_visible });
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_changed_markers(
    mut commands: Commands,
    zone_changers: Query<Entity, With<ZoneChanged>>,
    level_changers: Query<Entity, With<LevelChanged>>,
) {
    for entity in zone_changers.iter() {
        commands.entity(entity).remove::<ZoneChanged>();
    }
    for entity in level_changers.iter() {
        commands.entity(entity).remove::<LevelChanged>();
    }
}
