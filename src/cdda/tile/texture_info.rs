use crate::prelude::{MeshInfo, Transform2d};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct TextureInfo {
    pub(crate) mesh_info: MeshInfo,
    pub(crate) image_path: PathBuf,
    pub(crate) transform2d: Transform2d,
}
