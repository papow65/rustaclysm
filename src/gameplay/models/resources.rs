use crate::gameplay::Appearance;
use bevy::platform_support::collections::HashMap;
use bevy::prelude::{Mesh3d, Resource};
use cdda_json_files::SpriteNumber;
use std::path::PathBuf;

#[derive(Default, Resource)]
pub(super) struct AppearanceCache(pub(super) HashMap<PathBuf, Appearance>);

#[derive(Default, Resource)]
pub(super) struct MeshCaches {
    pub(super) horizontal_planes: HashMap<SpriteNumber, Mesh3d>,
    pub(super) vertical_planes: HashMap<SpriteNumber, Mesh3d>,
    pub(super) cuboids: HashMap<SpriteNumber, Mesh3d>,
}
