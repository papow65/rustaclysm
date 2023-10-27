use crate::prelude::{AssetState, OvermapBuffer, OvermapBufferPath, Overzone, SavPath};
use bevy::prelude::{AssetServer, Assets, Handle, Resource};
use bevy::utils::HashMap;

#[derive(Resource)]
pub(crate) struct OvermapBufferManager {
    sav_path: SavPath,
    all: HashMap<Handle<OvermapBuffer>, Overzone>,
}

impl OvermapBufferManager {
    pub(crate) fn new(sav_path: SavPath) -> Self {
        Self {
            sav_path,
            all: HashMap::new(),
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
                AssetState::Available { asset }
            } else {
                AssetState::Loading
            }
        } else {
            AssetState::Nonexistent
        }
    }

    pub(crate) fn overzone(&mut self, handle: &Handle<OvermapBuffer>) -> Overzone {
        self.all.get(handle).copied().expect("Known overmap buffer")
    }
}
