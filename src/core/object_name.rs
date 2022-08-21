use crate::prelude::*;
use bevy::prelude::Vec2;
use serde::Deserialize;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Deserialize)]
pub(crate) struct ObjectName(String);

impl ObjectName {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }

    pub(crate) fn to_label(&self) -> Label {
        Label::new(self.0.clone())
    }

    pub(crate) fn variants(&self) -> Vec<Self> {
        let mut result = vec![Self(self.0.clone() + "_season_summer"), self.clone()];
        if let Some(index) = self.0.rfind('_') {
            result.push(Self(self.0[..index].to_string()));
        }
        result
    }

    pub(crate) fn is_ground(&self) -> bool {
        self.0 == "t_grass" || self.0 == "t_dirt"
    }

    pub(crate) fn is_stairs_up(&self) -> bool {
        self.0.starts_with("t_stairs_up")
            || self.0.starts_with("t_wood_stairs_up")
            || self.0.starts_with("t_ladder_up")
            || self.0.starts_with("t_ramp_up")
            || self.0.starts_with("t_slope_up")
            || self.0.starts_with("t_gutter_downspout")
    }

    pub(crate) fn is_stairs_down(&self) -> bool {
        self.0.starts_with("t_stairs_down")
            || self.0.starts_with("t_wood_stairs_down")
            || self.0.starts_with("t_ladder_down")
            || self.0.starts_with("t_ramp_down")
            || self.0.starts_with("t_slope_down")
            || self.0.starts_with("t_gutter_downspout") // TODO
    }

    pub(crate) fn to_shape(
        &self,
        layer: SpriteLayer,
        transform2d: Transform2d,
        specifier: &ObjectSpecifier,
    ) -> ModelShape {
        if specifier == &ObjectSpecifier::ZoneLevel || self.0.starts_with("t_rock_floor") {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d,
            }
        } else if self.0.starts_with("t_rock")
            || self.0.starts_with("t_wall")
            || self.0.starts_with("t_brick_wall")
            || self.0.starts_with("t_concrete_wall")
            || self.0.starts_with("t_reinforced_glass")
            || self.0.starts_with("t_paper")
        {
            ModelShape::Cuboid {
                height: VERTICAL.f32(),
            }
        } else if self.0.starts_with("t_window")
            || self.0.starts_with("t_door")
            || self.0.starts_with("t_curtains")
            || self.0.starts_with("t_bars")
        {
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: Transform2d {
                    scale: Vec2::new(ADJACENT.f32(), VERTICAL.f32()),
                    offset: Vec2::new(0.0, 0.5 * (VERTICAL.f32() - ADJACENT.f32())),
                },
            }
        } else if self.0.starts_with("t_sewage_pipe") {
            ModelShape::Cuboid {
                height: ADJACENT.f32(),
            }
        } else if layer == SpriteLayer::Back {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d,
            }
        } else if 1.0 < transform2d.scale.x.max(transform2d.scale.y)
            || self.0.starts_with("t_fence")
            || self.0.starts_with("t_splitrail_fence")
            || self.0.starts_with("t_shrub")
            || self.0.starts_with("t_flower")
            || self.0.starts_with("f_plant")
            || self.0.starts_with("mon_")
        {
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d,
            }
        } else {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d,
            }
        }
    }
}
