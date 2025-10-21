use bevy::prelude::{Camera3d, Commands, PerspectiveProjection, Projection};

pub(super) fn create_camera(mut commands: Commands) {
    commands.spawn(((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            // more overview, less personal than the default
            fov: 0.3,
            ..PerspectiveProjection::default()
        }),
    ),));
}
