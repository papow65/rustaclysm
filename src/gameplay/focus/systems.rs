use crate::common::log_if_slow;
use crate::gameplay::{CameraBase, ExamineCursor, Focus, FocusState};
use bevy::prelude::{Camera3d, Query, Res, State, Transform, Vec3, Visibility, With, Without};
use std::time::Instant;

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_focus_cursor_visibility(
    focus_state: Res<State<FocusState>>,
    mut curors: Query<(&mut Visibility, &mut Transform), With<ExamineCursor>>,
) {
    let start = Instant::now();

    if let Ok((mut visibility, mut transform)) = curors.get_single_mut() {
        let examine_pos = matches!(**focus_state, FocusState::ExaminingPos(_));
        let examine_zone_level = matches!(**focus_state, FocusState::ExaminingZoneLevel(_));
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

    log_if_slow("update_focus_cursor_visibility", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_camera_base(
    focus: Focus,
    mut camera_bases: Query<&mut Transform, (With<CameraBase>, Without<Camera3d>)>,
) {
    let start = Instant::now();

    camera_bases.single_mut().translation = focus.offset();

    log_if_slow("update_camera", start);
}
