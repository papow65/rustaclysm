use crate::{ZoomDirection, ZoomDistance};
use bevy::prelude::{Resource, Vec2, Vec3};

#[derive(Debug, Default, Resource)]
pub(crate) struct CameraOffset {
    zoom_in_level: i32,
    /// left/right angle
    yaw_offset: f32,
    /// up/down angle
    pitch_offset: f32,
}

impl CameraOffset {
    const DEFAULT_OFFSET: Vec3 = Vec3::new(0.0, 28.4, 35.5);

    pub(crate) fn offset(&self) -> Vec3 {
        let length = Self::DEFAULT_OFFSET.length() * 0.75_f32.powi(self.zoom_in_level);
        Vec3::new(
            Self::DEFAULT_OFFSET.x + self.yaw_offset,
            Self::DEFAULT_OFFSET.y,
            Self::DEFAULT_OFFSET.z + self.pitch_offset,
        )
        .normalize()
            * length
    }

    pub(crate) const fn zoom(&mut self, zoom_direction: ZoomDirection) {
        self.zoom_in_level += if matches!(zoom_direction, ZoomDirection::In) {
            1
        } else {
            -1
        };
        //trace!("{:?}", (self.zoom_in_level);
    }

    pub(crate) const fn zoom_map_only(&self) -> bool {
        self.zoom_in_level < -10
    }

    pub(crate) const fn zoom_to_map(&mut self, zoom_distance: ZoomDistance) {
        self.zoom_in_level = if matches!(zoom_distance, ZoomDistance::Close) {
            -8
        } else {
            -12
        };
    }

    pub(crate) const fn zoom_to_tiles(&mut self, zoom_distance: ZoomDistance) {
        self.zoom_in_level = if matches!(zoom_distance, ZoomDistance::Close) {
            0
        } else {
            -7
        };
    }

    pub(crate) const fn zoom_tiles_only(&self) -> bool {
        -4 < self.zoom_in_level
    }

    pub(crate) fn adjust_angle(&mut self, delta: Vec2) {
        // Reduced sensitivity for better control
        const INPUT_SENSITIVITY: f32 = 0.125;
        let delta = delta * INPUT_SENSITIVITY;

        // An almost vertical angle makes sprites look more 2D.
        // An almost horizontal angle is bad for performance.
        self.pitch_offset = (self.pitch_offset - delta.y).clamp(-10.0, 20.0);

        // A large change makes using the direction keys weird.
        // A large change makes sprites look more 2D.
        let yaw_limit = 10.0 + self.pitch_offset / 4.0;
        self.yaw_offset = (self.yaw_offset - delta.x).clamp(-yaw_limit, yaw_limit);
    }

    pub(crate) const fn reset_angle(&mut self) {
        self.yaw_offset = 0.0;
        self.pitch_offset = 0.0;
    }
}
