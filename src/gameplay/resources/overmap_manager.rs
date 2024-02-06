use crate::prelude::{AssetState, Overmap, OvermapPath, Overzone, WorldPath};
use bevy::{
    prelude::{AssetId, AssetServer, Assets, Handle, Resource},
    utils::HashMap,
};

#[derive(Resource)]
pub(crate) struct OvermapManager {
    world_path: WorldPath,
    live_handles: Vec<Handle<Overmap>>,
    overzones: HashMap<AssetId<Overmap>, Overzone>,
}

impl OvermapManager {
    pub(crate) fn new(world_path: WorldPath) -> Self {
        Self {
            world_path,
            live_handles: Vec::new(),
            overzones: HashMap::new(),
        }
    }

    pub(crate) fn get_overmap<'a>(
        &mut self,
        asset_server: &AssetServer,
        overmap_assets: &'a Assets<Overmap>,
        overzone: Overzone,
    ) -> AssetState<'a, Overmap> {
        let overmap_path = OvermapPath::new(&self.world_path, overzone);
        if overmap_path.0.exists() {
            let overmap_handle = asset_server.load::<Overmap>(overmap_path.0.clone());
            let id = overmap_handle.id();
            self.live_handles.push(overmap_handle);
            self.overzones.insert(id, overzone);
            if let Some(asset) = overmap_assets.get(id) {
                AssetState::Available { asset }
            } else {
                AssetState::Loading
            }
        } else {
            AssetState::Nonexistent
        }
    }

    pub(crate) fn overzone(&mut self, handle: &AssetId<Overmap>) -> Option<Overzone> {
        //println!("Looking for {handle:?} in {:?}", self.overzones);
        self.overzones.get(handle).copied()
    }
}
