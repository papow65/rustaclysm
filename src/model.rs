use bevy::prelude::{Mesh, Quat, Transform, Vec2, Vec3};

use crate::mesh::MeshInfo;
use crate::resources::{SpriteLayer, SpriteNumber, TextureInfo, TileName};
use crate::unit::ADJACENT;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Transform2d {
    pub scale: Vec2,
    pub offset: Vec2,
}

impl Transform2d {
    fn to_scale(self) -> Vec3 {
        Vec3::new(self.scale.x, /*thickness*/ 1.0, self.scale.y)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SpriteOrientation {
    Horizontal,
    Vertical,
}

impl SpriteOrientation {
    fn to_rotation(self) -> Quat {
        match self {
            SpriteOrientation::Horizontal => Quat::from_rotation_y(0.5 * std::f32::consts::PI),
            SpriteOrientation::Vertical => {
                Quat::from_rotation_z(0.5 * std::f32::consts::PI)
                    * Quat::from_rotation_y(0.5 * std::f32::consts::PI)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpritePlane {
    pub orientation: SpriteOrientation,
    pub transform2d: Transform2d,
}

impl SpritePlane {
    fn to_transform(self, layer: &SpriteLayer) -> Transform {
        let translation = match self.orientation {
            SpriteOrientation::Horizontal => Vec3::new(
                /*back*/ self.transform2d.offset.y,
                /*up*/
                match layer {
                    SpriteLayer::Front => 0.01,
                    _ => 0.0,
                },
                /*right*/ self.transform2d.offset.x,
            ),
            SpriteOrientation::Vertical => Vec3::new(
                /*back*/
                match layer {
                    SpriteLayer::Front => -0.41,
                    SpriteLayer::Back => -0.4,
                },
                /*up*/ 0.02 + ADJACENT.f32() / 2.0 + self.transform2d.offset.y,
                /*right*/ self.transform2d.offset.x,
            ),
        };
        let rotation = self.orientation.to_rotation();
        let scale = self.transform2d.to_scale();
        Transform {
            translation,
            rotation,
            scale,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SpriteShape {
    Plane(SpritePlane),
    Cuboid(/*height*/ f32),
}

impl SpriteShape {
    fn to_transform(self, layer: &SpriteLayer) -> Transform {
        match self {
            SpriteShape::Plane(plane) => plane.to_transform(layer),
            SpriteShape::Cuboid(height) => Transform {
                scale: Vec3::new(ADJACENT.f32(), height, ADJACENT.f32()),
                ..Transform::default()
            },
        }
    }
}

#[derive(Debug)]
pub struct Model {
    pub shape: SpriteShape,
    pub layer: SpriteLayer,
    pub sprite_number: SpriteNumber,
    pub mesh_info: MeshInfo,
    pub texture_path: String,
}

impl Model {
    pub fn new(
        tile_name: &TileName,
        layer: SpriteLayer,
        sprite_number: SpriteNumber,
        texture_info: &TextureInfo,
    ) -> Self {
        Self {
            shape: tile_name.to_shape(layer, texture_info.transform2d),
            layer,
            sprite_number,
            mesh_info: texture_info.mesh_info,
            texture_path: texture_info.image_path.clone(),
        }
    }

    pub fn to_mesh(&self) -> Mesh {
        match self.shape {
            SpriteShape::Cuboid(_) => self.mesh_info.to_cube(),
            _ => self.mesh_info.to_plane(),
        }
    }

    pub fn to_transform(&self) -> Transform {
        self.shape.to_transform(&self.layer)
    }
}
