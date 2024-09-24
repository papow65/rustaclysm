use crate::gameplay::{AssetState, PathFor};
use bevy::prelude::{Asset, AssetId, AssetServer, Assets, Handle, Resource};
use bevy::utils::HashMap;

#[derive(Resource)]
pub(crate) struct AssetStorage<A: Asset, R: Clone + Copy> {
    live_handles: Vec<Handle<A>>,
    regions: HashMap<AssetId<A>, R>,
}

impl<A: Asset, R: Clone + Copy> AssetStorage<A, R> {
    pub(crate) fn handle<'a>(
        &mut self,
        asset_server: &AssetServer,
        assets: &'a Assets<A>,
        region: R,
        path: PathFor<A>,
    ) -> AssetState<'a, A> {
        if path.0.exists() {
            let overmap_handle = asset_server.load::<A>(path.0);
            let id = overmap_handle.id();
            self.live_handles.push(overmap_handle);
            self.regions.insert(id, region);
            if let Some(asset) = assets.get(id) {
                AssetState::Available { asset }
            } else {
                AssetState::Loading
            }
        } else {
            AssetState::Nonexistent
        }
    }

    pub(crate) fn region(&self, handle: &AssetId<A>) -> Option<R> {
        //println!("Looking for {handle:?} in {:?}", self.overzones);
        self.regions.get(handle).copied()
    }
}

impl<A: Asset, R: Clone + Copy> Default for AssetStorage<A, R> {
    fn default() -> Self {
        Self {
            live_handles: Vec::new(),
            regions: HashMap::new(),
        }
    }
}
