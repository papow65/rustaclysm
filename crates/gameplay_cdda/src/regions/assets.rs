use bevy::prelude::{Asset, TypePath};
use cdda_json_files::{Map, MapMemory, Overmap, OvermapBuffer};
use gameplay_location::{Overzone, ZoneLevel};
use serde::Deserialize;

pub(super) trait RegionAsset: Asset {
    type Region: Clone + Copy;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub struct MapAsset(pub Map);

impl RegionAsset for MapAsset {
    type Region = ZoneLevel;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub struct MapMemoryAsset(pub MapMemory);

impl RegionAsset for MapMemoryAsset {
    type Region = ZoneLevel;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub struct OvermapAsset(pub Overmap);

impl RegionAsset for OvermapAsset {
    type Region = Overzone;
}

#[derive(Debug, Deserialize, Asset, TypePath)]
pub struct OvermapBufferAsset(pub OvermapBuffer);

impl RegionAsset for OvermapBufferAsset {
    type Region = Overzone;
}
