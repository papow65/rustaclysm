use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct MeshInfo {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl MeshInfo {
    pub fn new(index: usize, width: usize, size: usize) -> Self {
        let index = index as f32;
        let size = size as f32;
        let width = width as f32; // tiles per row
        let height = (size / width).ceil() as f32; // tiles per column

        let x_min = (index % width) / width;
        let y_min = (index / width).floor() / height;

        Self {
            x_min,
            x_max: x_min + 1.0 / width,
            y_min,
            y_max: y_min + 1.0 / height,
        }
    }

    // Based on bevy_render-0.7.0/src/mesh/shape/mod.rs - line 194-223
    pub fn to_plane(self) -> Mesh {
        let extent = 0.5;
        let vertices = [
            ([extent, 0.0, -extent], [self.x_min, self.y_max]),
            ([extent, 0.0, extent], [self.x_min, self.y_min]),
            ([-extent, 0.0, extent], [self.x_max, self.y_min]),
            ([-extent, 0.0, -extent], [self.x_max, self.y_max]),
        ];

        let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        for (position, uv) in &vertices {
            positions.push(*position);
            uvs.push(*uv);
        }
        let normals = vec![[0.0, 1.0, 0.0]; 4];

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

        mesh
    }

    // Based on bevy_render-0.7.0/src/mesh/shape/mod.rs - line 61-120
    // Does not include the bottom face or the back face
    pub fn to_cube(self) -> Mesh {
        let vertices = [
            // right
            ([-0.5, 0.0, 0.5], [0., 0., 1.0], [self.x_min, self.y_max]),
            ([0.5, 0.0, 0.5], [0., 0., 1.0], [self.x_max, self.y_max]),
            ([0.5, 1.0, 0.5], [0., 0., 1.0], [self.x_max, self.y_min]),
            ([-0.5, 1.0, 0.5], [0., 0., 1.0], [self.x_min, self.y_min]),
            // left
            ([-0.5, 1.0, -0.5], [0., 0., -1.0], [self.x_max, self.y_min]),
            ([0.5, 1.0, -0.5], [0., 0., -1.0], [self.x_min, self.y_min]),
            ([0.5, 0.0, -0.5], [0., 0., -1.0], [self.x_min, self.y_max]),
            ([-0.5, 0.0, -0.5], [0., 0., -1.0], [self.x_max, self.y_max]),
            // front
            ([-0.5, 0.0, 0.5], [-1.0, 0., 0.], [self.x_max, self.y_max]),
            ([-0.5, 1.0, 0.5], [-1.0, 0., 0.], [self.x_max, self.y_min]),
            ([-0.5, 1.0, -0.5], [-1.0, 0., 0.], [self.x_min, self.y_min]),
            ([-0.5, 0.0, -0.5], [-1.0, 0., 0.], [self.x_min, self.y_max]),
            // top
            ([0.5, 1.0, -0.5], [0., 1.0, 0.], [self.x_min, self.y_min]),
            ([-0.5, 1.0, -0.5], [0., 1.0, 0.], [self.x_min, self.y_max]),
            ([-0.5, 1.0, 0.5], [0., 1.0, 0.], [self.x_max, self.y_max]),
            ([0.5, 1.0, 0.5], [0., 1.0, 0.], [self.x_max, self.y_min]),
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
            4, 5, 6, 6, 7, 4, // left
            8, 9, 10, 10, 11, 8, // front
            12, 13, 14, 14, 15, 12, // top
        ]);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(indices));
        mesh
    }
}
