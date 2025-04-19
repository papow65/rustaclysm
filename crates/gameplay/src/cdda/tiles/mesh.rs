use crate::SpriteOrientation;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) struct MeshInfo {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl MeshInfo {
    pub(crate) fn new(index: u16, tiles_per_row: u16, size: u16) -> Self {
        // rounding up 'size / tiles_per_row'
        let tiles_per_column = size.div_ceil(tiles_per_row);

        assert!(0 < tiles_per_row, "At least one tile per row");
        assert!(0 < tiles_per_column, "At least one tile per column");

        let tile_width = 1.0 / f32::from(tiles_per_row);
        let tile_height = 1.0 / f32::from(tiles_per_column);

        let x_min = f32::from(index % tiles_per_row) * tile_width;
        let y_min = f32::from(index / tiles_per_row) * tile_height;

        Self {
            x_min,
            x_max: x_min + tile_width,
            y_min,
            y_max: y_min + tile_height,
        }
    }

    // Based on bevy_render-0.7.0/src/mesh/shape/mod.rs - line 194-223
    pub(crate) fn to_plane(self, orientation: SpriteOrientation) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]));

        let positions = match orientation {
            SpriteOrientation::Horizontal => vec![
                [-0.5, 0.0, 0.5],  // front left
                [-0.5, 0.0, -0.5], // back left
                [0.5, 0.0, -0.5],  // back right
                [0.5, 0.0, 0.5],   // front right
            ],
            SpriteOrientation::Vertical => vec![
                [-0.5, 0.0, 0.0], // bottom left
                [-0.5, 1.0, 0.0], // top left
                [0.5, 1.0, 0.0],  // top right
                [0.5, 0.0, 0.0],  // bottom right
            ],
        };
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 4]);

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![
                [self.x_min, self.y_max], // bottom left
                [self.x_min, self.y_min], // top left
                [self.x_max, self.y_min], // top right
                [self.x_max, self.y_max], // bottom right
            ],
        );
        mesh
    }

    // Based on bevy_render-0.7.0/src/mesh/shape/mod.rs - line 61-120
    // Does not include the bottom face or the back face
    pub(crate) fn to_cube(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_indices(Indices::U32(vec![
            0, 1, 2, 2, 3, 0, // right
            4, 5, 6, 6, 7, 4, // front
            8, 9, 10, 10, 11, 8, // left
            12, 13, 14, 14, 15, 12, // top
        ]));

        let positions = vec![
            // back -> right
            [0.5, 1.0, -0.5],
            [0.5, 1.0, 0.5],
            [0.5, 0.0, 0.5],
            [0.5, 0.0, -0.5],
            // front
            [-0.5, 0.0, 0.5],
            [0.5, 0.0, 0.5],
            [0.5, 1.0, 0.5],
            [-0.5, 1.0, 0.5],
            // left
            [-0.5, 0.0, 0.5],
            [-0.5, 1.0, 0.5],
            [-0.5, 1.0, -0.5],
            [-0.5, 0.0, -0.5],
            // top
            [-0.5, 1.0, -0.5],
            [-0.5, 1.0, 0.5],
            [0.5, 1.0, 0.5],
            [0.5, 1.0, -0.5],
        ];
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        let normals = vec![
            // back -> right
            [1.0, 0., 0.],
            [1.0, 0., 0.],
            [1.0, 0., 0.],
            [1.0, 0., 0.],
            // front
            [0., 0., 1.0],
            [0., 0., 1.0],
            [0., 0., 1.0],
            [0., 0., 1.0],
            // left
            [-1.0, 0., 0.],
            [-1.0, 0., 0.],
            [-1.0, 0., 0.],
            [-1.0, 0., 0.],
            // top
            [0., 1.0, 0.],
            [0., 1.0, 0.],
            [0., 1.0, 0.],
            [0., 1.0, 0.],
        ];
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        let uvs = vec![
            // back -> right
            [self.x_max, self.y_min],
            [self.x_min, self.y_min],
            [self.x_min, self.y_max],
            [self.x_max, self.y_max],
            // front
            [self.x_min, self.y_max],
            [self.x_max, self.y_max],
            [self.x_max, self.y_min],
            [self.x_min, self.y_min],
            // left
            [self.x_max, self.y_max],
            [self.x_max, self.y_min],
            [self.x_min, self.y_min],
            [self.x_min, self.y_max],
            // top
            [self.x_min, self.y_min],
            [self.x_min, self.y_max],
            [self.x_max, self.y_max],
            [self.x_max, self.y_min],
        ];
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}
