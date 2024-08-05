use crate::gameplay::{
    Distance, ModelShape, ObjectCategory, SpriteLayer, SpriteOrientation, Transform2d,
};
use bevy::prelude::Vec2;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize)]
pub(crate) struct ObjectId(Arc<str>);

impl ObjectId {
    pub(crate) fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub(crate) fn suffix(&self, name: &str) -> Self {
        Self((String::from(&*self.0) + name).into())
    }

    pub(crate) fn truncate(&self) -> Self {
        Self(
            String::from(&*self.0)
                .replace("_isolated", "")
                .replace("_end_south", "")
                .replace("_end_west", "")
                .replace("_ne", "")
                .replace("_end_north", "")
                .replace("_ns", "")
                .replace("_es", "")
                .replace("_nes", "")
                .replace("_end_east", "")
                .replace("_wn", "")
                .replace("_ew", "")
                .replace("_new", "")
                .replace("_sw", "")
                .replace("_nsw", "")
                .replace("_esw", "")
                .replace("_nesw", "")
                .into(),
        )
    }

    pub(crate) fn fallback_name(&self) -> String {
        String::from(&*self.0)
    }

    pub(crate) fn is_moving_deep_water_zone(&self) -> bool {
        self.0.starts_with("river_")
    }

    pub(crate) fn is_still_deep_water_zone(&self) -> bool {
        self.0.starts_with("lake_")
    }

    pub(crate) fn is_grassy_zone(&self) -> bool {
        &*self.0 == "field" || self.0.starts_with("forest")
    }

    pub(crate) fn is_road_zone(&self) -> bool {
        self.0.starts_with("road_")
    }

    pub(crate) fn is_ground(&self) -> bool {
        &*self.0 == "t_grass" || &*self.0 == "t_dirt"
    }

    pub(crate) fn to_shape(
        &self,
        layer: SpriteLayer,
        transform2d: &Transform2d,
        category: &ObjectCategory,
    ) -> ModelShape {
        if category == &ObjectCategory::ZoneLevel
            || self.0.starts_with("t_rock_floor")
            || self.0.starts_with("t_rock_roof")
        {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        } else if self.0.starts_with("t_rock")
            || self.0.starts_with("t_wall")
            || self.0.starts_with("t_brick_wall")
            || self.0.starts_with("t_concrete_wall")
            || self.0.starts_with("t_reinforced_glass")
            || self.0.starts_with("t_paper")
            || self.0.starts_with("t_soil")
        {
            ModelShape::Cuboid {
                height: Distance::VERTICAL.meter_f32(),
            }
        } else if self.0.starts_with("t_window")
            || self.0.starts_with("t_door")
            || self.0.starts_with("t_curtains")
            || self.0.starts_with("t_bars")
        {
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: Transform2d {
                    scale: Vec2::new(
                        Distance::ADJACENT.meter_f32(),
                        Distance::VERTICAL.meter_f32(),
                    ),
                    offset: Vec2::ZERO,
                },
            }
        } else if self.0.starts_with("t_sewage_pipe") {
            ModelShape::Cuboid {
                height: Distance::ADJACENT.meter_f32(),
            }
        } else if self.0.starts_with("mon_") {
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: transform2d.clone(),
            }
        } else if layer == SpriteLayer::Back {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        } else if 1.0 < transform2d.scale.x.max(transform2d.scale.y)
            || self.0.starts_with("t_fence")
            || self.0.starts_with("t_chainfence")
            || self.0.starts_with("t_chaingate")
            || self.0.starts_with("t_splitrail_fence")
            || self.0.starts_with("t_shrub")
            || self.0.starts_with("t_flower")
            || self.0.starts_with("f_plant")
            || self.0.starts_with("t_grass_long")
            || self.0.starts_with("t_grass_tall")
        {
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: transform2d.clone(),
            }
        } else {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        }
    }
}
