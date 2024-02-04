use crate::prelude::SpriteOrientation;
use bevy::render::{
    mesh::{Indices, Mesh, PrimitiveTopology},
    render_asset::RenderAssetUsages,
};

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) struct MeshInfo {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl MeshInfo {
    pub(crate) fn new(index: usize, tiles_per_row: usize, size: usize) -> Self {
        // rounding up 'size / tiles_per_row'
        let tiles_per_column = (size + tiles_per_row - 1) / tiles_per_row;

        let tile_width = 1.0 / tiles_per_row as f32;
        let tile_height = 1.0 / tiles_per_column as f32;

        let x_min = (index % tiles_per_row) as f32 * tile_width;
        let y_min = (index / tiles_per_row) as f32 * tile_height;

        Self {
            x_min,
            x_max: x_min + tile_width,
            y_min,
            y_max: y_min + tile_height,
        }
    }

    // Based on bevy_render-0.7.0/src/mesh/shape/mod.rs - line 194-223
    pub(crate) fn to_plane(self, orientation: SpriteOrientation) -> Mesh {
        let extent = 0.5;
        let corners = match orientation {
            SpriteOrientation::Horizontal => [
                [-extent, 0.0, extent],
                [-extent, 0.0, -extent],
                [extent, 0.0, -extent],
                [extent, 0.0, extent],
            ],
            SpriteOrientation::Vertical => [
                [-extent, 0.0, 0.0],
                [-extent, 1.0, 0.0],
                [extent, 1.0, 0.0],
                [extent, 0.0, 0.0],
            ],
        };

        let vertices = [
            (corners[0], [self.x_min, self.y_max]),
            (corners[1], [self.x_min, self.y_min]),
            (corners[2], [self.x_max, self.y_min]),
            (corners[3], [self.x_max, self.y_max]),
        ];

        let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        for (position, uv) in &vertices {
            positions.push(*position);
            uvs.push(*uv);
        }
        let normals = vec![[0.0, 1.0, 0.0]; 4];

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        mesh
    }

    // Based on bevy_render-0.7.0/src/mesh/shape/mod.rs - line 61-120
    // Does not include the bottom face or the back face
    pub(crate) fn to_cube(self) -> Mesh {
        let vertices = [
            // back -> right
            ([0.5, 1.0, -0.5], [1.0, 0., 0.], [self.x_max, self.y_min]),
            ([0.5, 1.0, 0.5], [1.0, 0., 0.], [self.x_min, self.y_min]),
            ([0.5, 0.0, 0.5], [1.0, 0., 0.], [self.x_min, self.y_max]),
            ([0.5, 0.0, -0.5], [1.0, 0., 0.], [self.x_max, self.y_max]),
            // front
            ([-0.5, 0.0, 0.5], [0., 0., 1.0], [self.x_min, self.y_max]),
            ([0.5, 0.0, 0.5], [0., 0., 1.0], [self.x_max, self.y_max]),
            ([0.5, 1.0, 0.5], [0., 0., 1.0], [self.x_max, self.y_min]),
            ([-0.5, 1.0, 0.5], [0., 0., 1.0], [self.x_min, self.y_min]),
            // left
            ([-0.5, 0.0, 0.5], [-1.0, 0., 0.], [self.x_max, self.y_max]),
            ([-0.5, 1.0, 0.5], [-1.0, 0., 0.], [self.x_max, self.y_min]),
            ([-0.5, 1.0, -0.5], [-1.0, 0., 0.], [self.x_min, self.y_min]),
            ([-0.5, 0.0, -0.5], [-1.0, 0., 0.], [self.x_min, self.y_max]),
            // top
            ([-0.5, 1.0, -0.5], [0., 1.0, 0.], [self.x_min, self.y_min]),
            ([-0.5, 1.0, 0.5], [0., 1.0, 0.], [self.x_min, self.y_max]),
            ([0.5, 1.0, 0.5], [0., 1.0, 0.], [self.x_max, self.y_max]),
            ([0.5, 1.0, -0.5], [0., 1.0, 0.], [self.x_max, self.y_min]),
        ];

        let mut positions = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut uvs = Vec::with_capacity(24);

        for (position, normal, uv) in &vertices {
            positions.push(*position);
            normals.push(*normal);
            uvs.push(*uv);
        }

        let indices = Indices::U32(vec![
            0, 1, 2, 2, 3, 0, // right
            4, 5, 6, 6, 7, 4, // front
            8, 9, 10, 10, 11, 8, // left
            12, 13, 14, 14, 15, 12, // top
        ]);

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(indices));
        mesh
    }
}
