use crate::prelude::{OvermapBuffer, OvermapBufferPath, Overzone, SavPath};
use bevy::prelude::{AssetServer, Handle, Resource};
use bevy::utils::HashMap;

#[derive(Resource)]
pub(crate) struct OvermapBufferManager {
    sav_path: SavPath,
    loading: HashMap<Handle<OvermapBuffer>, Overzone>,
}

impl OvermapBufferManager {
    pub(crate) fn new(sav_path: SavPath) -> Self {
        Self {
            sav_path,
            loading: HashMap::new(),
        }
    }

    pub(crate) fn start_loading(&mut self, asset_server: &AssetServer, overzone: Overzone) {
        if !self.loading.values().any(|o| *o == overzone) {
            let handle = asset_server
                .load::<OvermapBuffer, _>(OvermapBufferPath::new(&self.sav_path, overzone).0);
            self.loading.insert(handle, overzone);
        }
    }

    pub(crate) fn finish_loading(&mut self, handle: &Handle<OvermapBuffer>) -> Overzone {
        self.loading.remove(handle).expect("Loading overmap buffer")
    }

    pub(crate) fn loaded(&self) -> bool {
        self.loading.is_empty()
    }
}
