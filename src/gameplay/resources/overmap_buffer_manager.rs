use crate::prelude::{AssetState, OvermapBuffer, OvermapBufferPath, Overzone, SavPath};
use bevy::prelude::{AssetServer, Assets, Handle, Resource};
use bevy::utils::HashMap;

#[derive(Resource)]
pub(crate) struct OvermapBufferManager {
    sav_path: SavPath,
    all: HashMap<Handle<OvermapBuffer>, Overzone>,
    loading: Vec<Handle<OvermapBuffer>>,
}

impl OvermapBufferManager {
    pub(crate) fn new(sav_path: SavPath) -> Self {
        Self {
            sav_path,
            all: HashMap::new(),
            loading: Vec::new(),
        }
    }

    pub(crate) fn get<'a>(
        &mut self,
        asset_server: &AssetServer,
        overmap_buffer_assets: &'a Assets<OvermapBuffer>,
        overzone: Overzone,
    ) -> AssetState<'a, OvermapBuffer> {
        let path = OvermapBufferPath::new(&self.sav_path, overzone);
        if path.0.exists() {
            let handle = asset_server.load(path.0);
            self.all.insert(handle.clone(), overzone);
            if let Some(asset) = overmap_buffer_assets.get(&handle) {
                self.mark_loaded(&handle);
                AssetState::Available { asset }
            } else {
                self.loading.push(handle);
                AssetState::Loading
            }
        } else {
            AssetState::Nonexistent
        }
    }

    pub(crate) fn mark_loaded(&mut self, handle: &Handle<OvermapBuffer>) -> Overzone {
        self.loading.retain(|h| h != handle);
        self.all.get(handle).copied().expect("Known overmap buffer")
    }

    pub(crate) fn loaded(&self) -> bool {
        self.loading.is_empty()
    }
}
