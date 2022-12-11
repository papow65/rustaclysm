use super::log_if_slow;
use crate::prelude::*;
use bevy::math::Quat;
use bevy::prelude::*;
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_location(
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
pub(crate) fn update_transforms(mut obstacles: Query<(&Pos, &mut Transform), Changed<Pos>>) {
    let start = Instant::now();

    for (&pos, mut transform) in obstacles.iter_mut() {
        transform.translation = pos.vec3();
    }

    log_if_slow("update_transforms", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_hidden_item_visibility(
    mut hidden_items: Query<&mut Visibility, Without<Pos>>,
    removed_positions: RemovedComponents<Pos>,
) {
    let start = Instant::now();

    // TODO use update_visualization
    for entity in removed_positions.iter() {
        if let Ok(mut visibility) = hidden_items.get_mut(entity) {
            visibility.is_visible = false;
        }
    }

    log_if_slow("update_visibility_for_hidden_items", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_cursor_visibility_on_player_change(
    mut curors: Query<(&mut Visibility, &mut Transform), With<ExamineCursor>>,
    players: Query<&Player, Changed<Player>>,
) {
    let start = Instant::now();

    if let Ok(player) = players.get_single() {
        if let Ok((mut visible, mut transform)) = curors.get_single_mut() {
            let examine_pos = matches!(player.state, PlayerActionState::ExaminingPos(_));
            let examine_zone_level =
                matches!(player.state, PlayerActionState::ExaminingZoneLevel(_));
            visible.is_visible = examine_pos || examine_zone_level;
            transform.scale = if examine_zone_level {
                Vec3::splat(24.0)
            } else {
                Vec3::ONE
            };
        }
    }

    log_if_slow("update_cursor_visibility_on_player_change", start);
}

fn update_material(
    commands: &mut Commands,
    children: &Children,
    child_items: &Query<&Appearance, (With<Parent>, Without<Pos>)>,
    last_seen: &LastSeen,
) {
    for &child in children.iter() {
        if let Ok(child_appearance) = child_items.get(child) {
            commands
                .entity(child)
                .insert(child_appearance.material(last_seen));
        }
    }
}

fn update_visualization(
    commands: &mut Commands,
    explored: &mut Explored,
    currently_visible: &CurrentlyVisible,
    focus: &Focus,
    pos: Pos,
    visibility: &mut Visibility,
    last_seen: &mut LastSeen,
    speed: Option<&Speed>,
    children: &Children,
    child_items: &Query<&Appearance, (With<Parent>, Without<Pos>)>,
) {
    let previously_seen = last_seen.clone();

    let visible = currently_visible.can_see(pos);
    // TODO check if there is enough light
    last_seen.update(&visible);

    if last_seen != &LastSeen::Never {
        if last_seen != &previously_seen {
            explored.mark_pos_seen(pos);

            // TODO select an appearance based on amount of perceived light
            update_material(commands, children, child_items, last_seen);
        }

        // The player character can see things not shown to the player, like the top of a tower when walking next to it.
        let level_shown = focus.is_shown(pos.level);
        visibility.is_visible = level_shown && last_seen.shown(speed.is_some());
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_item_move(
    mut commands: Commands,
    envir: Envir,
    mut explored: ResMut<Explored>,
    mut moved_items: Query<
        (
            &Pos,
            &mut Visibility,
            &mut LastSeen,
            Option<&Speed>,
            &Children,
        ),
        Changed<Pos>,
    >,
    child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    players: Query<(&Pos, &Player)>,
) {
    let start = Instant::now();

    if moved_items.iter().peekable().peek().is_some() {
        let (&player_pos, player) = players.single();
        let focus = Focus::new(player, player_pos);
        let currently_visible = envir.currently_visible(player_pos); // does not depend on focus

        for (&pos, mut visibility, mut last_seen, speed, children) in moved_items.iter_mut() {
            update_visualization(
                &mut commands,
                &mut explored,
                &currently_visible,
                &focus,
                pos,
                &mut visibility,
                &mut last_seen,
                speed,
                children,
                &child_items,
            );
        }
    }

    log_if_slow("update_visualization_on_item_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_focus_move(
    mut commands: Commands,
    envir: Envir,
    mut explored: ResMut<Explored>,
    mut last_focus: Local<Focus>,
    mut items: Query<(
        &Pos,
        &mut Visibility,
        &mut LastSeen,
        Option<&Speed>,
        &Children,
    )>,
    child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    players: Query<(&Pos, &Player)>,
) {
    let start = Instant::now();

    if let Ok((&player_pos, player)) = players.get_single() {
        let focus = Focus::new(player, player_pos);
        if focus != *last_focus {
            let currently_visible = envir.currently_visible(player_pos); // does not depend on focus

            for (&pos, mut visibility, mut last_seen, speed, children) in items.iter_mut() {
                update_visualization(
                    &mut commands,
                    &mut explored,
                    &currently_visible,
                    &focus,
                    pos,
                    &mut visibility,
                    &mut last_seen,
                    speed,
                    children,
                    &child_items,
                );
            }

            *last_focus = focus;
        }
    }

    log_if_slow("update_visualization_on_focus_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_damaged_characters(
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
            commands.spawn(Message::new(message));
        } else {
            let message = format!("{attacker} kills {label}", attacker = damage.attacker);
            commands.spawn(Message::new(message));
            transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
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
pub(crate) fn update_damaged_items(
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
            commands.spawn(Message::new(message));
        } else {
            commands
                .entity(item)
                .insert(Hurdle(3.0))
                .remove::<Obstacle>();
            commands.spawn(Message::new(format!(
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
pub(crate) fn update_camera(
    changed_players: Query<(&Pos, &Player), Changed<Player>>,
    mut camera_bases: Query<&mut Transform, (With<CameraBase>, Without<Camera3d>)>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let start = Instant::now();

    if let Ok((&pos, player)) = changed_players.get_single() {
        for mut transform in camera_bases.iter_mut() {
            transform.translation = match player.state {
                PlayerActionState::ExaminingPos(target) => target.vec3() - pos.vec3(),
                PlayerActionState::ExaminingZoneLevel(target) => {
                    target.base_pos().vec3() - pos.vec3() + Vec3::new(11.5, 0.0, 11.5)
                }
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
