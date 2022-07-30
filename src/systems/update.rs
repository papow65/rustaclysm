use super::log_if_slow;
use crate::prelude::*;
use bevy::math::Quat;
use bevy::prelude::*;
use std::time::Instant;

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
pub fn update_transforms(mut obstacles: Query<(&Pos, &mut Transform), Changed<Pos>>) {
    let start = Instant::now();

    for (&pos, mut transform) in obstacles.iter_mut() {
        transform.translation = pos.vec3();
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
    children: Option<&Children>,
    player_pos: Pos,
    child_items: &mut Query<&mut Visibility, (With<Parent>, Without<Pos>)>,
) {
    // TODO make something only invisibile when it would overlap with the player FOV
    // TODO partially show obstacles that overlap with the player and his nbors

    let is_visible = pos.1 == player_pos.1 || (0 <= pos.1 && pos.1 < player_pos.1);

    if let Some(children) = children {
        for &child in children.iter() {
            if let Ok(mut child_visibility) = child_items.get_mut(child) {
                child_visibility.is_visible = is_visible;
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_visibility_on_item_y_change(
    mut moved_items: Query<(&Pos, Option<&Children>), With<PosYChanged>>,
    mut child_items: Query<&mut Visibility, (With<Parent>, Without<Pos>)>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    let &player_pos = players.single();
    for (&pos, children) in moved_items.iter_mut() {
        update_visibility(pos, children, player_pos, &mut child_items);
    }

    log_if_slow("update_visibility_on_item_y_change", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_visibility_on_player_y_change(
    mut items: Query<(&Pos, Option<&Children>)>,
    mut child_items: Query<&mut Visibility, (With<Parent>, Without<Pos>)>,
    moved_players: Query<(&Pos, &Player), Or<(With<PosYChanged>, Changed<Player>)>>,
) {
    let start = Instant::now();

    if let Ok((&player_pos, player)) = moved_players.get_single() {
        let reference = match player.state {
            PlayerActionState::Examining(pos) => pos,
            _ => player_pos,
        };
        for (&pos, children) in items.iter_mut() {
            update_visibility(pos, children, reference, &mut child_items);
        }
    }

    log_if_slow("update_visibility_on_player_y_change", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_cursor_visibility_on_player_change(
    mut curors: Query<(&mut Visibility, &GlobalTransform), With<ExamineCursor>>,
    players: Query<&Player, Changed<Player>>,
) {
    let start = Instant::now();

    if let Ok(player) = players.get_single() {
        if let Ok((mut visible, gt)) = curors.get_single_mut() {
            visible.is_visible = matches!(player.state, PlayerActionState::Examining(_));
            println!("{:?}", gt);
        }
    }

    log_if_slow("update_cursor_visibility_on_player_change", start);
}

fn update_material(
    commands: &mut Commands,
    envir: &Envir,
    _entity: Entity,
    pos: Pos,
    prev_player_visible: &mut PlayerVisible,
    children: Option<&Children>,
    player_pos: Pos,
    child_items: &mut Query<&Appearance, (With<Parent>, Without<Pos>)>,
) {
    let player_visible = envir.can_see(player_pos, pos);
    if player_visible != *prev_player_visible {
        *prev_player_visible = player_visible;

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
    mut moved_items: Query<(Entity, &Pos, &mut PlayerVisible, Option<&Children>), Changed<Pos>>,
    mut child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    let &player_pos = players.single();
    for (entity, &pos, mut prev_player_visible, children) in moved_items.iter_mut() {
        update_material(
            &mut commands,
            &envir,
            entity,
            pos,
            &mut prev_player_visible,
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
    mut items: Query<(Entity, &Pos, &mut PlayerVisible, Option<&Children>)>,
    mut child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    moved_players: Query<&Pos, (With<Player>, Changed<Pos>)>,
) {
    let start = Instant::now();

    if let Ok(&player_pos) = moved_players.get_single() {
        for (entity, &pos, mut prev_player_visible, children) in items.iter_mut() {
            update_material(
                &mut commands,
                &envir,
                entity,
                pos,
                &mut prev_player_visible,
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
) {
    let start = Instant::now();
    if let Ok(&player_pos) = moved_players.get_single() {
        tiles.par_for_each_mut(64, |(parent, mut sprite)| {
            let &pos = tile_parents.get(parent.get()).unwrap();
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
                "{} hits {label} for {} ({prev} -> {curr})",
                damage.attacker, damage.amount
            );
            commands.spawn().insert(Message::new(message));
        } else {
            let message = format!("{attacker} kills {label}", attacker = damage.attacker);
            commands.spawn().insert(Message::new(message));
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
            commands.spawn().insert(Message::new(message));
        } else {
            commands
                .entity(item)
                .insert(Hurdle(3.0))
                .remove::<Obstacle>();
            commands.spawn().insert(Message::new(format!(
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
    changed_players: Query<(&Pos, &Player), Changed<Player>>,
    mut camera_bases: Query<&mut Transform, (With<CameraBase>, Without<Camera3d>)>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let start = Instant::now();

    if let Ok((&pos, player)) = changed_players.get_single() {
        for mut transform in camera_bases.iter_mut() {
            transform.translation = match player.state {
                PlayerActionState::Examining(target) => target.vec3() - pos.vec3(),
                _ => Vec3::ZERO,
            };
        }

        for mut transform in cameras.iter_mut() {
            let view_direction = Vec3::new(
                0.0 * player.camera_distance,
                4.0 * player.camera_distance,
                5.0 * player.camera_distance,
            );
            transform.translation = view_direction;
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }

    log_if_slow("update_camera", start);
}
