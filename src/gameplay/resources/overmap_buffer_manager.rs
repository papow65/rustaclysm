use crate::prelude::{
    AssetState, OvermapAsset, OvermapBuffer, OvermapBufferPath, Overzone, SavPath,
};
use bevy::{
    prelude::{AssetId, AssetServer, Assets, Handle, Resource},
    utils::HashMap,
};

#[derive(Resource)]
pub(crate) struct OvermapBufferManager {
    sav_path: SavPath,
    live_handles: Vec<Handle<OvermapAsset>>,
    overzones: HashMap<AssetId<OvermapAsset>, Overzone>,
}

impl OvermapBufferManager {
    pub(crate) fn new(sav_path: SavPath) -> Self {
        Self {
            sav_path,
            live_handles: Vec::new(),
            overzones: HashMap::new(),
        }
    }

    pub(crate) fn get<'a>(
        &mut self,
        asset_server: &AssetServer,
        overmap_buffer_assets: &'a Assets<OvermapAsset>,
        overzone: Overzone,
    ) -> AssetState<'a, OvermapBuffer> {
        let path = OvermapBufferPath::new(&self.sav_path, overzone);
        if path.0.exists() {
            let handle = asset_server.load(path.0.clone());
            let id = handle.id();
            self.overzones.insert(id, overzone);
            self.live_handles.push(handle);
            if let Some(asset) = overmap_buffer_assets.get(id) {
                AssetState::Available {
                    asset: asset.buffer(&path),
                }
            } else {
                AssetState::Loading
            }
        } else {
            AssetState::Nonexistent
        }
    }

    pub(crate) fn overzone(&mut self, handle: &AssetId<OvermapAsset>) -> Option<Overzone> {
        self.overzones.get(handle).copied()
    }
}
