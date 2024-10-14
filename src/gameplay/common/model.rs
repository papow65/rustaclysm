use crate::gameplay::{MeshInfo, ObjectCategory, ObjectDefinition, SpriteLayer, TextureInfo};
use bevy::prelude::{AlphaMode, Mesh, Transform, Vec2, Vec3};
use cdda_json_files::SpriteNumber;
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
        layer: &SpriteLayer,
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
        layer: &SpriteLayer,
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
        definition: &ObjectDefinition,
        layer: SpriteLayer,
        transform2d: &Transform2d,
    ) -> Self {
        if definition.category == ObjectCategory::ZoneLevel
            || definition.id.starts_with("t_rock_floor")
            || definition.id.starts_with("t_rock_roof")
        {
            Self::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: transform2d.clone(),
            }
        } else if definition.id.contains("solar_panel") {
            Self::Plane {
                orientation: SpriteOrientation::Horizontal,
                transform2d: Transform2d {
                    scale: transform2d.scale,
                    offset: transform2d.offset
                        + Vec2::new(0.0, -0.5 * Distance::ADJACENT.meter_f32()),
                },
            }
        } else if definition.id.starts_with("t_rock")
            || definition.id.starts_with("t_wall")
            || definition.id.starts_with("t_brick_wall")
            || definition.id.starts_with("t_concrete_wall")
            || definition.id.starts_with("t_reinforced_glass")
            || definition.id.starts_with("t_paper")
            || definition.id.starts_with("t_soil")
        {
            Self::Cuboid {
                height: Distance::VERTICAL.meter_f32(),
            }
        } else if definition.id.starts_with("t_window")
            || definition.id.starts_with("t_door")
            || definition.id.starts_with("t_curtains")
            || definition.id.starts_with("t_bars")
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
        } else if definition.id.starts_with("t_sewage_pipe") {
            Self::Cuboid {
                height: Distance::ADJACENT.meter_f32(),
            }
        } else if definition.id.starts_with("mon_") {
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
            || definition.id.starts_with("t_fence")
            || definition.id.starts_with("t_chainfence")
            || definition.id.starts_with("t_chaingate")
            || definition.id.starts_with("t_splitrail_fence")
            || definition.id.starts_with("t_shrub")
            || definition.id.starts_with("t_flower")
            || definition.id.starts_with("f_plant")
            || definition.id.starts_with("t_grass_long")
            || definition.id.starts_with("t_grass_tall")
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

    fn to_transform(&self, layer: &SpriteLayer, vertical_offset: f32) -> Transform {
        match self {
            Self::Plane {
                orientation,
                transform2d,
            } => transform2d.to_transform(*orientation, layer, vertical_offset),
            Self::Cuboid { height } => Transform {
                scale: match *layer {
                    SpriteLayer::Front => 1.0,
                    SpriteLayer::Back => 0.98,
                } * Vec3::new(
                    Distance::ADJACENT.meter_f32(),
                    *height,
                    Distance::ADJACENT.meter_f32(),
                ),
                translation: match *layer {
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
        definition: &ObjectDefinition,
        layer: SpriteLayer,
        sprite_number: SpriteNumber,
        texture_info: &TextureInfo,
    ) -> Self {
        Self {
            shape: ModelShape::derive(definition, layer, &texture_info.transform2d),
            layer,
            sprite_number,
            mesh_info: texture_info.mesh_info,
            texture_path: texture_info.image_path.clone(),
            vertical_offset: definition.category.vertical_offset(&layer),
            alpha_mode: definition.alpha_mode(),
        }
    }

    pub(crate) fn to_mesh(&self) -> Mesh {
        match self.shape {
            ModelShape::Plane { orientation, .. } => self.mesh_info.to_plane(orientation),
            ModelShape::Cuboid { .. } => self.mesh_info.to_cube(),
        }
    }

    pub(crate) fn to_transform(&self) -> Transform {
        self.shape.to_transform(&self.layer, self.vertical_offset)
    }
}
