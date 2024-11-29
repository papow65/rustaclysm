use crate::gameplay::cdda::paths::OvermapPath;
use crate::gameplay::cdda::region_assets::AssetStorage;
use crate::gameplay::{ActiveSav, AssetState, OvermapAsset, Overzone};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};

#[derive(SystemParam)]
pub(crate) struct OvermapManager<'w> {
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<OvermapAsset>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<OvermapAsset>>,
}

impl OvermapManager<'_> {
    pub(crate) fn get(&mut self, overzone: Overzone) -> AssetState<OvermapAsset> {
        let path = OvermapPath::new(&self.active_sav.world_path(), overzone);
        self.storage
            .handle(&self.asset_server, &self.assets, overzone, path)
    }

    pub(crate) fn overzone(&self, handle: &AssetId<OvermapAsset>) -> Option<Overzone> {
        self.storage.region(handle)
    }
}
