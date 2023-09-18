use crate::prelude::{AssetState, Map, MapPath, Submap, SubzoneLevel, WorldPath, ZoneLevel};
use bevy::prelude::{AssetServer, Assets, Handle, Resource};

#[derive(Resource)]
pub(crate) struct MapManager {
    world_path: WorldPath,
    loading: Vec<Handle<Map>>,
}

impl MapManager {
    pub(crate) fn new(world_path: WorldPath) -> Self {
        Self {
            world_path,
            loading: Vec::new(),
        }
    }

    pub(crate) fn get_zone_level<'a>(
        &mut self,
        asset_server: &AssetServer,
        map_assets: &'a Assets<Map>,
        zone_level: ZoneLevel,
    ) -> AssetState<'a, Map> {
        let map_path = MapPath::new(&self.world_path, zone_level);
        if map_path.0.exists() {
            let map_handle = asset_server.load(map_path.0);
            if let Some(map) = map_assets.get(&map_handle) {
                self.mark_loaded(&map_handle);
                AssetState::Available { asset: map }
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
        subzone_level: SubzoneLevel,
    ) -> AssetState<'a, Submap> {
        let zone_level = ZoneLevel::from(subzone_level);
        match self.get_zone_level(asset_server, map_assets, zone_level) {
            AssetState::Available { asset: map } => AssetState::Available {
                asset: &map.0[subzone_level.index()],
            },
            AssetState::Loading => AssetState::Loading,
            AssetState::Nonexistent => AssetState::Nonexistent,
        }
    }

    pub(crate) fn mark_loaded(&mut self, handle: &Handle<Map>) {
        self.loading.retain(|h| h != handle);
    }

    pub(crate) fn loaded(&self) -> bool {
        self.loading.is_empty()
    }
}
