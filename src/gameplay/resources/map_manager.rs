use crate::prelude::{Map, MapPath, Paths, Submap, SubzoneLevel, ZoneLevel};
use bevy::prelude::{AssetServer, Assets, Handle, Resource};

#[derive(Debug)]
pub(crate) enum AssetState<'a, T> {
    Available { asset: &'a T },
    Loading,
    Nonexistent,
}

#[derive(Default, Resource)]
pub(crate) struct MapManager {
    pub(crate) loading: Vec<Handle<Map>>,
}

impl MapManager {
    pub(crate) fn get_zone_level<'a>(
        &mut self,
        asset_server: &AssetServer,
        map_assets: &'a Assets<Map>,
        paths: &Paths,
        zone_level: ZoneLevel,
    ) -> AssetState<'a, Map> {
        let map_path = MapPath::new(&paths.world_path(), zone_level);
        if map_path.0.exists() {
            let map_handle = asset_server.load(map_path.0);
            if let Some(map) = map_assets.get(&map_handle) {
                AssetState::Available { asset: &map }
            } else {
                self.loading.push(map_handle);
                AssetState::Loading
            }
        } else {
            AssetState::Nonexistent
        }
    }

    pub(crate) fn get_subzone_level<'a>(
        &mut self,
        asset_server: &AssetServer,
        map_assets: &'a Assets<Map>,
        paths: &Paths,
        subzone_level: SubzoneLevel,
    ) -> AssetState<'a, Submap> {
        let zone_level = ZoneLevel::from(subzone_level);
        match self.get_zone_level(asset_server, map_assets, paths, zone_level) {
            AssetState::Available { asset: map } => AssetState::Available {
                asset: &map.0[subzone_level.index()],
            },
            AssetState::Loading => AssetState::Loading,
            AssetState::Nonexistent => AssetState::Nonexistent,
        }
    }
}
