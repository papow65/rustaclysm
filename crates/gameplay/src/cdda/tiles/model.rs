use crate::{MeshInfo, ObjectCategory, SpriteLayer, TextureInfo};
use bevy::prelude::{AlphaMode, Mesh, Transform, Vec2, Vec3};
use cdda_json_files::{InfoId, SpriteNumber, TerrainInfo, UntypedInfoId};
use std::path::PathBuf;
use units::Distance;

#[derive(Debug)]
pub(crate) struct Layers<T> {
    pub(crate) base: T,
    pub(crate) overlay: Option<T>,
}

impl<T> Layers<T> {
    pub(crate) fn map<U>(self, f: impl Fn(T) -> U) -> Layers<U> {
        let Self { base, overlay } = self;
        Layers {
            base: f(base),
            overlay: overlay.map(f),
        }
    }

    pub(crate) fn map_mut<U>(self, mut f: impl FnMut(T) -> U) -> Layers<U> {
        let Self { base, overlay } = self;
        Layers {
            base: f(base),
            overlay: overlay.map(f),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Transform2d {
    pub(crate) scale: Vec2,
    pub(crate) offset: Vec2,
}

impl Transform2d {
    fn to_transform(
        &self,
        orientation: SpriteOrientation,
        layer: SpriteLayer,
        vertical_offset: f32,
    ) -> Transform {
        Transform {
            translation: self.to_translation(orientation, layer, vertical_offset),
            scale: self.to_scale(orientation),
            ..Transform::default()
        }
    }

    fn to_translation(
        &self,
        orientation: SpriteOrientation,
        layer: SpriteLayer,
        vertical_offset: f32,
    ) -> Vec3 {
        match orientation {
            SpriteOrientation::Horizontal => Vec3::new(
                self.offset.x,   // right
                vertical_offset, // up
                -self.offset.y,
            ),
            SpriteOrientation::Vertical => Vec3::new(
                self.offset.x,   // right
                vertical_offset, // up
                match layer {
                    SpriteLayer::Front => 0.41,
                    SpriteLayer::Back => 0.4,
                }, // front
            ),
        }
    }

    const fn to_scale(&self, orientation: SpriteOrientation) -> Vec3 {
        match orientation {
            SpriteOrientation::Horizontal => {
                Vec3::new(
                    self.scale.x,
                    1.0, // thickness
                    self.scale.y,
                )
            }
            SpriteOrientation::Vertical => {
                Vec3::new(
                    self.scale.x,
                    self.scale.y,
                    1.0, // thickness
                )
            }
        }
    }
}

impl Default for Transform2d {
    fn default() -> Self {
        Self {
            scale: Vec2::ONE,
            offset: Vec2::ZERO,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SpriteOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ModelShape {
    Plane {
        orientation: SpriteOrientation,
        transform2d: Transform2d,
    },
    Cuboid {
        height: f32,
    },
}

impl ModelShape {
    fn derive(
        info_id: &UntypedInfoId,
        category: ObjectCategory,
        layer: SpriteLayer,
        transform2d: &Transform2d,
    ) -> Self {
        if category == ObjectCategory::ZoneLevel
            || info_id.starts_with("t_rock_floor")
            || info_id.starts_with("t_rock_roof")
        {
            Self::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        } else if info_id.contains("solar_panel") {
            Self::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: Transform2d {
                    scale: transform2d.scale,
                    offset: transform2d.offset
                        + Vec2::new(0.0, -0.5 * Distance::ADJACENT.meter_f32()),
                },
            }
        } else if info_id.starts_with("t_rock")
            || info_id.starts_with("t_wall")
            || info_id.starts_with("t_brick_wall")
            || info_id.starts_with("t_concrete_wall")
            || info_id.starts_with("t_reinforced_glass")
            || info_id.starts_with("t_paper")
            || info_id.starts_with("t_soil")
        {
            Self::Cuboid {
                height: Distance::VERTICAL.meter_f32(),
            }
        } else if info_id.starts_with("t_window")
            || info_id.starts_with("t_door")
            || info_id.starts_with("t_curtains")
            || info_id.starts_with("t_bars")
        {
            Self::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: Transform2d {
                    scale: Vec2::new(
                        Distance::ADJACENT.meter_f32(),
                        Distance::VERTICAL.meter_f32(),
                    ),
                    offset: Vec2::ZERO,
                },
            }
        } else if info_id.starts_with("t_sewage_pipe") {
            Self::Cuboid {
                height: Distance::ADJACENT.meter_f32(),
            }
        } else if info_id.starts_with("mon_") {
            Self::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: transform2d.clone(),
            }
        } else if layer == SpriteLayer::Back {
            Self::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        } else if 1.0 < transform2d.scale.x.max(transform2d.scale.y)
            || info_id.starts_with("t_fence")
            || info_id.starts_with("t_chainfence")
            || info_id.starts_with("t_chaingate")
            || info_id.starts_with("t_splitrail_fence")
            || info_id.starts_with("t_shrub")
            || info_id.starts_with("t_flower")
            || info_id.starts_with("f_plant")
            || info_id.starts_with("t_grass_long")
            || info_id.starts_with("t_grass_tall")
        {
            Self::Plane {
                orientation: SpriteOrientation::Vertical,
                transform2d: transform2d.clone(),
            }
        } else {
            Self::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        }
    }

    fn to_transform(&self, layer: SpriteLayer, vertical_offset: f32) -> Transform {
        match self {
            Self::Plane {
                orientation,
                transform2d,
            } => transform2d.to_transform(*orientation, layer, vertical_offset),
            Self::Cuboid { height } => Transform {
                scale: match layer {
                    SpriteLayer::Front => 1.0,
                    SpriteLayer::Back => 0.98,
                } * Vec3::new(
                    Distance::ADJACENT.meter_f32(),
                    *height,
                    Distance::ADJACENT.meter_f32(),
                ),
                translation: match layer {
                    SpriteLayer::Front => Vec3::ZERO,
                    SpriteLayer::Back => Vec3::new(0.0, 0.0, -0.01),
                },
                ..Transform::default()
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) shape: ModelShape,
    pub(crate) layer: SpriteLayer,
    pub(crate) sprite_number: SpriteNumber,
    pub(crate) mesh_info: MeshInfo,
    pub(crate) texture_path: PathBuf,
    pub(crate) vertical_offset: f32,
    pub(crate) alpha_mode: AlphaMode,
}

impl Model {
    pub(crate) fn new(
        info_id: &UntypedInfoId,
        category: ObjectCategory,
        layer: SpriteLayer,
        sprite_number: SpriteNumber,
        texture_info: &TextureInfo,
    ) -> Self {
        let alpha_mode = if category == ObjectCategory::Terrain
            && InfoId::<TerrainInfo>::from(info_id.clone()).is_ground()
        {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        };

        Self {
            shape: ModelShape::derive(info_id, category, layer, &texture_info.transform2d),
            layer,
            sprite_number,
            mesh_info: texture_info.mesh_info,
            texture_path: texture_info.image_path.clone(),
            vertical_offset: category.vertical_offset(layer),
            alpha_mode,
        }
    }

    pub(crate) fn to_mesh(&self) -> Mesh {
        match self.shape {
            ModelShape::Plane { orientation, .. } => self.mesh_info.to_plane(orientation),
            ModelShape::Cuboid { .. } => self.mesh_info.to_cube(),
        }
    }

    pub(crate) fn to_transform(&self) -> Transform {
        self.shape.to_transform(self.layer, self.vertical_offset)
    }
}
