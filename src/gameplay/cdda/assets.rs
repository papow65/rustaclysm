use bevy::prelude::{Asset, TypePath};
use cdda_json_files::{Map, MapMemory, Overmap, OvermapBuffer};
use serde::Deserialize;

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct MapAsset(pub(crate) Map);

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct MapMemoryAsset(pub(crate) MapMemory);

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct OvermapAsset(pub(crate) Overmap);

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct OvermapBufferAsset(pub(crate) OvermapBuffer);
