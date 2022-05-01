use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::math::Quat;
use bevy::prelude::*;
use bevy::render::camera::Camera3d;
use bevy::tasks::ComputeTaskPool;
use std::time::Instant;

use super::super::components::{
    Corpse, Damage, Faction, Floor, Health, Hurdle, Integrity, Label, LogDisplay, Message,
    Obstacle, Player, PlayerVisible, Pos, PosYChanged, Stairs, StatusDisplay,
};
use super::super::resources::{Envir, Location, Timeouts};
use super::super::units::{Speed, VERTICAL};

use super::{log_if_slow, Appearance};

#[allow(clippy::needless_pass_by_value)]
pub fn update_location(
    mut location: ResMut<Location>,
    changed_positions: Query<(Entity, &Pos), Changed<Pos>>,
    removed_positions: RemovedComponents<Pos>,
) {
    let start = Instant::now();

    for (entity, &pos) in changed_positions.iter() {
        location.update(entity, Some(pos));
    }

    for entity in removed_positions.iter() {
        location.update(entity, None);
    }

    log_if_slow("update_location", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_transforms(
    mut obstacles: Query<
        (
            &Pos,
            &mut Transform,
            Option<&Floor>,
            Option<&Stairs>,
            Option<&Corpse>,
        ),
        (
            Or<(Changed<Pos>, Changed<Corpse>)>,
            Without<TextureAtlasSprite>,
        ),
    >,
) {
    let start = Instant::now();

    for (&pos, mut transform, floor, stair, corpse) in obstacles.iter_mut() {
        let vertical_height = if floor.is_some() {
            0.0
        } else if stair.is_some() {
            VERTICAL.f32()
        } else if corpse.is_some() {
            0.01
        } else {
            transform.scale.y
        };
        transform.translation = pos.vec3(vertical_height);
    }

    log_if_slow("update_transforms", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_visibility_for_hidden_items(
    mut hidden_items: Query<&mut Visibility, Without<Pos>>,
    removed_positions: RemovedComponents<Pos>,
) {
    let start = Instant::now();

    for entity in removed_positions.iter() {
        if let Ok(mut visibility) = hidden_items.get_mut(entity) {
            visibility.is_visible = true; // TODO false;
        }
    }

    log_if_slow("update_visibility_for_hidden_items", start);
}

fn update_visibility(
    pos: Pos,
    visibility: &mut Visibility,
    children: Option<&Children>,
    player_pos: Pos,
    child_items: &mut Query<&mut Visibility, (With<Parent>, Without<Pos>)>,
) {
    // TODO make something only invisibility when it would overlap with the player FOV
    // TODO partially show obstacles that overlap with the player and his nbors

    visibility.is_visible = pos.1 <= player_pos.1;

    if let Some(children) = children {
        for &child in children.iter() {
            if let Ok(mut child_visibility) = child_items.get_mut(child) {
                child_visibility.is_visible = visibility.is_visible;
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_visibility_on_item_y_change(
    mut commands: Commands,
    mut moved_items: Query<(Entity, &Pos, &mut Visibility, Option<&Children>), With<PosYChanged>>,
    mut child_items: Query<&mut Visibility, (With<Parent>, Without<Pos>)>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    let &player_pos = players.single();
    for (entity, &pos, mut visibility, children) in moved_items.iter_mut() {
        update_visibility(pos, &mut visibility, children, player_pos, &mut child_items);
        commands.entity(entity).remove::<PosYChanged>();
    }

    log_if_slow("update_visibility_on_item_y_change", start);
}

/// for both meshes and tiles
#[allow(clippy::needless_pass_by_value)]
pub fn update_visibility_on_player_y_change(
    mut commands: Commands,
    mut items: Query<(&Pos, &mut Visibility, Option<&Children>)>,
    mut child_items: Query<&mut Visibility, (With<Parent>, Without<Pos>)>,
    moved_players: Query<(Entity, &Pos), (With<Player>, With<PosYChanged>)>,
) {
    let start = Instant::now();

    if let Ok((player, &player_pos)) = moved_players.get_single() {
        for (&pos, mut visibility, children) in items.iter_mut() {
            update_visibility(pos, &mut visibility, children, player_pos, &mut child_items);
        }
        commands.entity(player).remove::<PosYChanged>();
    }

    log_if_slow("update_visibility_on_player_y_change", start);
}

fn update_material(
    commands: &mut Commands,
    envir: &Envir,
    entity: Entity,
    pos: Pos,
    prev_player_visible: &mut PlayerVisible,
    appearance: &Appearance,
    children: Option<&Children>,
    player_pos: Pos,
    child_items: &mut Query<&Appearance, (With<Parent>, Without<Pos>)>,
) {
    let player_visible = envir.can_see(player_pos, pos);
    if player_visible != *prev_player_visible {
        *prev_player_visible = player_visible;

        commands
            .entity(entity)
            .insert(appearance.material(player_visible));

        if let Some(children) = children {
            for &child in children.iter() {
                if let Ok(child_appearance) = child_items.get_mut(child) {
                    commands
                        .entity(child)
                        .insert(child_appearance.material(player_visible));
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_material_on_item_move(
    mut commands: Commands,
    envir: Envir,
    mut moved_items: Query<
        (
            Entity,
            &Pos,
            &mut PlayerVisible,
            &Appearance,
            Option<&Children>,
        ),
        Changed<Pos>,
    >,
    mut child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    let &player_pos = players.single();
    for (entity, &pos, mut prev_player_visible, appearance, children) in moved_items.iter_mut() {
        update_material(
            &mut commands,
            &envir,
            entity,
            pos,
            &mut prev_player_visible,
            appearance,
            children,
            player_pos,
            &mut child_items,
        );
    }

    log_if_slow("update_material_on_item_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_material_on_player_move(
    mut commands: Commands,
    envir: Envir,
    mut items: Query<(
        Entity,
        &Pos,
        &mut PlayerVisible,
        &Appearance,
        Option<&Children>,
    )>,
    mut child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    moved_players: Query<&Pos, (With<Player>, Changed<Pos>)>,
) {
    let start = Instant::now();

    if let Ok(&player_pos) = moved_players.get_single() {
        for (entity, &pos, mut prev_player_visible, appearance, children) in items.iter_mut() {
            update_material(
                &mut commands,
                &envir,
                entity,
                pos,
                &mut prev_player_visible,
                appearance,
                children,
                player_pos,
                &mut child_items,
            );
        }
    }

    log_if_slow("update_material_on_player_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_tile_color_on_player_move(
    envir: Envir,
    mut tiles: Query<(&Parent, &mut TextureAtlasSprite)>,
    tile_parents: Query<&Pos, With<Children>>,
    moved_players: Query<&Pos, (With<Player>, Changed<Pos>)>,
    pool: Res<ComputeTaskPool>,
) {
    let start = Instant::now();
    if let Ok(&player_pos) = moved_players.get_single() {
        tiles.par_for_each_mut(&pool, 64, |(parent, mut sprite)| {
            let &pos = tile_parents.get(parent.0).unwrap();
            sprite.color = envir.can_see(player_pos, pos).adjust(Color::WHITE);
        });
    }

    log_if_slow("update_tile_color_on_player_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_damaged_characters(
    mut commands: Commands,
    mut characters: Query<(Entity, &Label, &mut Health, &Damage, &mut Transform), With<Faction>>,
) {
    let start = Instant::now();

    for (character, label, mut health, damage, mut transform) in characters.iter_mut() {
        let prev = format!("{health}", health = *health);
        if health.apply(damage) {
            let curr = format!("{health}", health = *health);
            let message = format!(
                "{} hits {} for {} ({} -> {})",
                damage.attacker, label, damage.amount, prev, curr
            );
            commands.spawn_bundle(Message::new(message));
        } else {
            commands.spawn_bundle(Message::new(format!(
                "{attacker} kills {label}",
                attacker = damage.attacker
            )));
            transform.rotation = Quat::from_rotation_y(0.5 * std::f32::consts::PI)
                * Quat::from_rotation_x(-0.5 * std::f32::consts::PI);
            commands
                .entity(character)
                .insert(Corpse)
                .insert(Label::new("corpse"))
                .remove::<Health>()
                .remove::<Obstacle>();
        }

        commands.entity(character).remove::<Damage>();
    }

    log_if_slow("update_damaged_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_damaged_items(
    mut commands: Commands,
    mut windows: Query<(Entity, &Label, &mut Integrity, &Damage, Option<&Children>)>,
) {
    let start = Instant::now();

    for (item, label, mut integrity, damage, _children) in windows.iter_mut() {
        let prev = format!("{integrity}", integrity = *integrity);
        if integrity.apply(damage) {
            let curr = format!("{integrity}", integrity = *integrity);
            let message = format!(
                "{attacker} hits {label} ({prev} -> {curr})",
                attacker = damage.attacker
            );
            commands.spawn_bundle(Message::new(message));
        } else {
            commands
                .entity(item)
                .insert(Hurdle(3.0))
                .remove::<Obstacle>();
            commands.spawn_bundle(Message::new(format!(
                "{} breaks {}",
                damage.attacker, label
            )));
        }
        /*
        Causes a crash
        TODO What was this supposed to do?
        if let Some(children) = children {
            for &child in children.iter() {
                commands.entity(child).despawn();
            }
        }*/

        commands.entity(item).remove::<Damage>();
    }

    log_if_slow("update_damaged_items", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_camera(
    changed_players: Query<&Player, Changed<Player>>,
    mut cameras: Query<(&Camera3d, &mut Transform)>,
) {
    let start = Instant::now();

    if let Ok(player) = changed_players.get_single() {
        for (_camera, mut transform) in cameras.iter_mut() {
            let translation = Vec3::new(
                0.0 * player.camera_distance,
                4.0 * player.camera_distance,
                5.0 * player.camera_distance,
            );
            transform.translation = translation;
            transform.look_at(-translation, Vec3::Y);
        }
    }

    log_if_slow("update_camera", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_log(
    mut logs: Query<&mut Text, With<LogDisplay>>,
    messages: Query<&Message>,
    changed: Query<&Message, Changed<Message>>,
) {
    let start = Instant::now();

    let mut new_messages = false;
    for message in changed.iter() {
        println!("{string}", string = message.0);
        new_messages = true;
    }
    if !new_messages {
        return;
    }

    let log = messages
        .iter()
        .map(|m| format!("{string}\n", string = m.0))
        .collect::<Vec<String>>();

    logs.iter_mut().next().unwrap().sections[0].value = log
        [std::cmp::max(log.len() as isize - 20, 0) as usize..log.len()]
        .iter()
        .map(String::as_str)
        .collect::<String>();

    log_if_slow("update_log", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_status_fps(
    diagnostics: Res<Diagnostics>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if diagnostics.is_changed() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Precision of 0.1s
                // Padding to 6 characters, aligned right
                status_displays.iter_mut().next().unwrap().sections[0].value =
                    format!("{average:>6.1} fps\n");
            }
        }
    }

    log_if_slow("update_status_fps", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_status_time(
    timeouts: Res<Timeouts>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    let tenth_seconds = timeouts.time().0 / 100;
    let seconds = tenth_seconds / 10;
    let minutes = seconds / 10;
    let hours = minutes / 60;

    status_displays.iter_mut().next().unwrap().sections[1].value = format!(
        "{:#02}:{:#02}:{:#02}.{}\n",
        hours,
        minutes % 60,
        seconds % 60,
        tenth_seconds % 10
    );

    log_if_slow("update_status_time", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_status_health(
    health: Query<&Health, (With<Player>, Changed<Health>)>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Some(health) = health.iter().next() {
        status_displays.iter_mut().next().unwrap().sections[2].value =
            format!("{:>3} health\n", format!("{}", health));
    }

    log_if_slow("update_status_health", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_status_speed(
    speed: Query<&Speed, (With<Player>, Changed<Speed>)>,
    mut status_displays: Query<&mut Text, With<StatusDisplay>>,
) {
    let start = Instant::now();

    if let Some(speed) = speed.iter().next() {
        status_displays.iter_mut().next().unwrap().sections[3].value =
            format!("{:>10}\n", format!("{}", speed.h));
    }

    log_if_slow("update_status_speed", start);
}
