use crate::{ActiveSav, AssetState, OvermapAsset, OvermapPath, region_assets::AssetStorage};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};
use gameplay_location::Overzone;

#[derive(SystemParam)]
pub struct OvermapManager<'w> {
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<OvermapAsset>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<OvermapAsset>>,
}

impl OvermapManager<'_> {
    pub fn load(&mut self, overzone: Overzone) -> AssetState<'_, OvermapAsset> {
        let path = OvermapPath::new(&self.active_sav.world_path(), overzone);
        self.storage
            .handle(&self.asset_server, &self.assets, overzone, path)
    }

    #[must_use]
    pub fn overzone(&self, handle: &AssetId<OvermapAsset>) -> Option<Overzone> {
        self.storage.region(handle)
    }
}
