use crate::prelude::*;
use bevy::prelude::*;
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_transforms(
    mut obstacles: Query<(&Pos, &mut Transform), Changed<Pos>>,
) {
    let start = Instant::now();

    for (&pos, mut transform) in &mut obstacles {
        transform.translation = pos.vec3();
    }

    log_if_slow("update_transforms", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_hidden_item_visibility(
    mut hidden_items: Query<&mut Visibility, Without<Pos>>,
    mut removed_positions: RemovedComponents<Pos>,
) {
    let start = Instant::now();

    // TODO use update_visualization
    for entity in removed_positions.read() {
        if let Ok(mut visibility) = hidden_items.get_mut(entity) {
            *visibility = Visibility::Hidden;
        }
    }

    log_if_slow("update_visibility_for_hidden_items", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_cursor_visibility_on_player_change(
    player_action_state: Res<PlayerActionState>,
    mut curors: Query<(&mut Visibility, &mut Transform), With<ExamineCursor>>,
) {
    let start = Instant::now();

    if let Ok((mut visibility, mut transform)) = curors.get_single_mut() {
        let examine_pos = matches!(*player_action_state, PlayerActionState::ExaminingPos(_));
        let examine_zone_level = matches!(
            *player_action_state,
            PlayerActionState::ExaminingZoneLevel(_)
        );
        *visibility = if examine_pos || examine_zone_level {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        transform.scale = if examine_zone_level {
            Vec3::splat(24.0)
        } else {
            Vec3::ONE
        };
    }

    log_if_slow("update_cursor_visibility_on_player_change", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_visualization_on_weather_change(
    clock: Clock,
    player_action_state: Res<PlayerActionState>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut session: GameplaySession,
    mut last_viewing_disttance: Local<Option<usize>>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    if session.is_changed() {
        *last_viewing_disttance = None;
    }

    let player_pos = players.single();
    let viewing_distance =
        CurrentlyVisible::viewing_distance(&clock, &player_action_state, player_pos.level);
    if *last_viewing_disttance != viewing_distance {
        *last_viewing_disttance = viewing_distance;

        // Handled by update_visualization_on_focus_move next frame
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("update_visualization_on_weather_change", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_camera_base(
    player_state: Res<PlayerActionState>,
    players: Query<&Pos, With<Player>>,
    mut camera_bases: Query<&mut Transform, (With<CameraBase>, Without<Camera3d>)>,
) {
    let start = Instant::now();

    let pos = players.single();

    for mut transform in &mut camera_bases {
        transform.translation = match *player_state {
            PlayerActionState::ExaminingPos(target) => target.vec3() - pos.vec3(),
            PlayerActionState::ExaminingZoneLevel(target) => {
                target.base_pos().vec3() - pos.vec3() + Vec3::new(11.5, 0.0, 11.5)
            }
            _ => Vec3::ZERO,
        };
    }

    log_if_slow("update_camera", start);
}

#[cfg(debug_assertions)]
#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn check_items(
    item_parents: Query<Option<&Parent>, Or<(With<Amount>, With<Containable>)>>,
) {
    assert!(
        item_parents.iter().all(|o| o.is_some()),
        "All items should have a parent"
    );
}
