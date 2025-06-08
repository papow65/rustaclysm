use crate::{AssetState, PathFor, region_assets::RegionAsset};
use bevy::platform::collections::HashMap;
use bevy::prelude::{AssetId, AssetServer, Assets, Handle, Resource};

#[derive(Resource)]
pub(super) struct AssetStorage<A: RegionAsset> {
    live_handles: Vec<Handle<A>>,
    regions: HashMap<AssetId<A>, A::Region>,
}

impl<A: RegionAsset> AssetStorage<A> {
    pub(super) fn handle<'a>(
        &mut self,
        asset_server: &AssetServer,
        assets: &'a Assets<A>,
        region: A::Region,
        path: PathFor<A>,
    ) -> AssetState<'a, A> {
        if path.0.exists() {
            let region_handle = asset_server.load::<A>(path.0);
            let id = region_handle.id();
            self.live_handles.push(region_handle);
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

    pub(super) fn region(&self, handle: &AssetId<A>) -> Option<A::Region> {
        //trace!("Looking for {handle:?} in {:?}", self.overzones);
        self.regions.get(handle).copied()
    }
}

impl<A: RegionAsset> Default for AssetStorage<A> {
    fn default() -> Self {
        Self {
            live_handles: Vec::new(),
            regions: HashMap::default(),
        }
    }
}
