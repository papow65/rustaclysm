use crate::prelude::*;
use bevy::prelude::{AlphaMode, Mesh, Transform, Vec2, Vec3};
use std::path::PathBuf;

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

/// Everything to make a `PbrBundle`
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
            shape: definition
                .id
                .to_shape(layer, &texture_info.transform2d, &definition.category),
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
