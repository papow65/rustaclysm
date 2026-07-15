use bevy::prelude::Resource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoomDistance {
    Close,
    Far,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoomDirection {
    In,
    Out,
}

#[derive(Debug, Default, Resource)]
pub struct CameraZoom {
    zoom_in_level: i32,
}

impl CameraZoom {
    pub fn distance(&self) -> f32 {
        45.0 * 0.75_f32.powi(self.zoom_in_level)
    }

    pub const fn zoom(&mut self, zoom_direction: ZoomDirection) {
        self.zoom_in_level += if matches!(zoom_direction, ZoomDirection::In) {
            1
        } else {
            -1
        };
        //trace!("{:?}", (self.zoom_in_level);
    }

    pub const fn zoom_map_only(&self) -> bool {
        self.zoom_in_level < -10
    }

    pub const fn zoom_to_map(&mut self, zoom_distance: ZoomDistance) {
        self.zoom_in_level = if matches!(zoom_distance, ZoomDistance::Close) {
            -8
        } else {
            -12
        };
    }

    pub const fn zoom_to_tiles(&mut self, zoom_distance: ZoomDistance) {
        self.zoom_in_level = if matches!(zoom_distance, ZoomDistance::Close) {
            0
        } else {
            -7
        };
    }

    pub const fn zoom_tiles_only(&self) -> bool {
        -4 < self.zoom_in_level
    }
}
