use crate::Appearance;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Mesh3d, Resource};
use cdda_json_files::SpriteNumber;
use std::path::PathBuf;

#[derive(Default, Resource)]
pub(crate) struct AppearanceCache(pub(crate) HashMap<PathBuf, Appearance>);

#[derive(Default, Resource)]
pub(crate) struct MeshCaches {
    pub(crate) horizontal_planes: HashMap<SpriteNumber, Mesh3d>,
    pub(crate) vertical_planes: HashMap<SpriteNumber, Mesh3d>,
    pub(crate) cuboids: HashMap<SpriteNumber, Mesh3d>,
}
