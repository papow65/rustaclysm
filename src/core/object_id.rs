use crate::prelude::*;
use bevy::prelude::Vec2;
use serde::Deserialize;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Deserialize)]
pub(crate) struct ObjectId(String);

impl ObjectId {
    pub(crate) fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }

    pub(crate) fn to_fallback_label(&self) -> Label {
        Label::new(self.0.clone())
    }

    pub(crate) fn variants(&self) -> Vec<Self> {
        let mut result = vec![Self(self.0.clone() + "_season_summer"), self.clone()];
        if let Some(index) = self.0.rfind('_') {
            result.push(Self(self.0[..index].to_string()));
        }
        result
    }

    pub(crate) fn is_hidden_zone(&self) -> bool {
        self.0 == "open_air"
            || self.0 == "solid_earth"
            || self.0 == "deep_rock"
            || self.0 == "empty_rock"
    }

    pub(crate) fn is_ground(&self) -> bool {
        self.0 == "t_grass" || self.0 == "t_dirt"
    }

    pub(crate) fn to_shape(
        &self,
        layer: SpriteLayer,
        transform2d: &Transform2d,
        specifier: &ObjectSpecifier,
    ) -> ModelShape {
        if specifier == &ObjectSpecifier::ZoneLevel || self.0.starts_with("t_rock_floor") {
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
                height: Millimeter::VERTICAL.f32(),
            }
        } else if self.0.starts_with("t_window")
            || self.0.starts_with("t_door")
            || self.0.starts_with("t_curtains")
            || self.0.starts_with("t_bars")
        {
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: Transform2d {
                    scale: Vec2::new(Millimeter::ADJACENT.f32(), Millimeter::VERTICAL.f32()),
                    offset: Vec2::ZERO,
                },
            }
        } else if self.0.starts_with("t_sewage_pipe") {
            ModelShape::Cuboid {
                height: Millimeter::ADJACENT.f32(),
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
            assert_eq!(
                transform2d,
                &Transform2d::default(),
                "no standard transform for foreground of {:?}: {:?}",
                self,
                transform2d
            );
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        }
    }
}
