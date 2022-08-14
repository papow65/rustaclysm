use crate::prelude::*;
use bevy::prelude::{AlphaMode, Mesh, Quat, Transform, Vec2, Vec3};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Transform2d {
    pub scale: Vec2,
    pub offset: Vec2,
}

impl Transform2d {
    const fn to_scale(self) -> Vec3 {
        Vec3::new(self.scale.x, /*thickness*/ 1.0, self.scale.y)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpriteOrientation {
    Horizontal,
    Vertical,
}

impl SpriteOrientation {
    fn to_rotation(self) -> Quat {
        match self {
            Self::Horizontal => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Self::Vertical => {
                Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                    * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModelShape {
    Plane {
        orientation: SpriteOrientation,
        transform2d: Transform2d,
    },
    Cuboid {
        height: f32,
    },
}

impl ModelShape {
    fn to_transform(self, layer: &SpriteLayer, vertical_offset: f32) -> Transform {
        match self {
            Self::Plane {
                orientation,
                transform2d,
            } => {
                let translation = match orientation {
                    SpriteOrientation::Horizontal => Vec3::new(
                        /*back*/ transform2d.offset.y,
                        /*up*/
                        vertical_offset,
                        /*right*/ transform2d.offset.x,
                    ),
                    SpriteOrientation::Vertical => Vec3::new(
                        /*back*/
                        match layer {
                            SpriteLayer::Front => -0.41,
                            SpriteLayer::Back => -0.4,
                        },
                        /*up*/
                        vertical_offset + ADJACENT.f32() / 2.0 + transform2d.offset.y,
                        /*right*/ transform2d.offset.x,
                    ),
                };
                let rotation = orientation.to_rotation();
                let scale = transform2d.to_scale();
                Transform {
                    translation,
                    rotation,
                    scale,
                }
            }
            Self::Cuboid { height } => Transform {
                scale: Vec3::new(ADJACENT.f32(), height, ADJACENT.f32()),
                ..Transform::default()
            },
        }
    }
}

/** Everything to make a `PbrBundle` */
#[derive(Debug)]
pub struct Model {
    pub shape: ModelShape,
    pub layer: SpriteLayer,
    pub sprite_number: SpriteNumber,
    pub mesh_info: MeshInfo,
    pub texture_path: String,
    pub vertical_offset: f32,
    pub alpha_mode: AlphaMode,
}

impl Model {
    pub fn new(
        definition: &ObjectDefinition,
        layer: SpriteLayer,
        sprite_number: SpriteNumber,
        texture_info: &TextureInfo,
    ) -> Self {
        Self {
            shape: definition
                .name
                .to_shape(layer, texture_info.transform2d, &definition.specifier),
            layer,
            sprite_number,
            mesh_info: texture_info.mesh_info,
            texture_path: texture_info.image_path.clone(),
            vertical_offset: definition.specifier.vertical_offset(&layer),
            alpha_mode: definition.alpha_mode(),
        }
    }

    pub fn to_mesh(&self) -> Mesh {
        match self.shape {
            ModelShape::Plane { .. } => self.mesh_info.to_plane(),
            ModelShape::Cuboid { .. } => self.mesh_info.to_cube(),
        }
    }

    pub fn to_transform(&self) -> Transform {
        self.shape.to_transform(&self.layer, self.vertical_offset)
    }
}
