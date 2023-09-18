use crate::prelude::{AssetState, Overmap, OvermapPath, Overzone, WorldPath};
use bevy::{
    prelude::{AssetServer, Assets, Handle, Resource},
    utils::HashMap,
};

#[derive(Resource)]
pub(crate) struct OvermapManager {
    world_path: WorldPath,
    all: HashMap<Handle<Overmap>, Overzone>,
    loading: Vec<Handle<Overmap>>,
}

impl OvermapManager {
    pub(crate) fn new(world_path: WorldPath) -> Self {
        Self {
            world_path,
            all: HashMap::new(),
            loading: Vec::new(),
        }
    }

    pub(crate) fn get_overmap<'a>(
        &mut self,
        asset_server: &AssetServer,
        overmap_assets: &'a Assets<Overmap>,
        overzone: Overzone,
    ) -> AssetState<'a, Overmap> {
        let map_path = OvermapPath::new(&self.world_path, overzone);
        if map_path.0.exists() {
            let overmap_handle = asset_server.load(map_path.0);
            self.all.insert(overmap_handle.clone(), overzone);
            if let Some(overmap) = overmap_assets.get(&overmap_handle) {
                self.mark_loaded(&overmap_handle);
                AssetState::Available { asset: overmap }
            } else {
                self.loading.push(overmap_handle);
                AssetState::Loading
            }
        } else {
            AssetState::Nonexistent
        }
    }

    pub(crate) fn mark_loaded(&mut self, handle: &Handle<Overmap>) -> Overzone {
        self.loading.retain(|h| h != handle);
        self.all.get(handle).copied().expect("Known overmap")
    }

    pub(crate) fn loaded(&self) -> bool {
        self.loading.is_empty()
    }
}
