use crate::gameplay::{Level, Pos, Region, ZoneLevel};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Camera, GlobalTransform, Rect, Single, Vec2};

/// Region visible on the camera
#[derive(SystemParam)]
pub(crate) struct VisibleRegion<'w> {
    camera: Single<'w, (&'static Camera, &'static GlobalTransform)>,
}

impl VisibleRegion<'_> {
    pub(super) fn global_transform(&self) -> GlobalTransform {
        *self.camera.1
    }

    pub(super) fn calculate_all(&self) -> Region {
        self.calculate(&Level::ALL)
    }

    pub(super) fn calculate_ground(&self) -> Region {
        self.calculate(&Level::GROUNDS)
    }

    fn calculate(&self, levels: &[Level]) -> Region {
        let (camera, &global_transform) = *self.camera;

        let Some(Rect {
            min: corner_min,
            max: corner_max,
        }) = camera.logical_viewport_rect()
        else {
            return Region::new(&Vec::new());
        };

        let mut zone_levels = Vec::new();
        let floor: fn(f32) -> f32 = f32::floor;
        let ceil: fn(f32) -> f32 = f32::ceil;
        for &level in levels {
            for (corner, round_x, round_z) in [
                (Vec2::new(corner_min.x, corner_min.y), floor, floor),
                (Vec2::new(corner_min.x, corner_max.y), floor, ceil),
                (Vec2::new(corner_max.x, corner_min.y), ceil, floor),
                (Vec2::new(corner_max.x, corner_max.y), ceil, ceil),
            ] {
                let Ok(ray) = camera.viewport_to_world(&global_transform, corner) else {
                    continue;
                };

                let ray_distance = (level.f32() - ray.origin.y) / ray.direction.y;
                // The camera only looks forward.
                if 0.0 < ray_distance {
                    let floor = ray.get_point(ray_distance);
                    //trace!("{:?}", ((level, ray_distance, floor.x, floor.z));
                    zone_levels.push(ZoneLevel::from(Pos {
                        x: round_x(floor.x) as i32,
                        level,
                        z: round_z(floor.z) as i32,
                    }));
                }
            }
        }

        Region::new(&zone_levels)
    }
}
