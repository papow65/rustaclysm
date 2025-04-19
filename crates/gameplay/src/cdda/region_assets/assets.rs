use crate::{Overzone, ZoneLevel};
use bevy::prelude::{Asset, TypePath};
use cdda_json_files::{Map, MapMemory, Overmap, OvermapBuffer};
use serde::Deserialize;

pub(super) trait RegionAsset: Asset {
    type Region: Clone + Copy;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct MapAsset(pub(crate) Map);

impl RegionAsset for MapAsset {
    type Region = ZoneLevel;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct MapMemoryAsset(pub(crate) MapMemory);

impl RegionAsset for MapMemoryAsset {
    type Region = ZoneLevel;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct OvermapAsset(pub(crate) Overmap);

impl RegionAsset for OvermapAsset {
    type Region = Overzone;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub(crate) struct OvermapBufferAsset(pub(crate) OvermapBuffer);

impl RegionAsset for OvermapBufferAsset {
    type Region = Overzone;
}
