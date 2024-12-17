use crate::gameplay::cdda::paths::OvermapBufferPath;
use crate::gameplay::cdda::region_assets::AssetStorage;
use crate::gameplay::{ActiveSav, AssetState, OvermapBufferAsset, Overzone};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};

#[derive(SystemParam)]
pub(crate) struct OvermapBufferManager<'w> {
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<OvermapBufferAsset>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<OvermapBufferAsset>>,
}

impl OvermapBufferManager<'_> {
    pub(crate) fn load(&mut self, overzone: Overzone) -> AssetState<OvermapBufferAsset> {
        let path = OvermapBufferPath::new(&self.active_sav.sav_path(), overzone);
        self.storage
            .handle(&self.asset_server, &self.assets, overzone, path)
    }

    pub(crate) fn overzone(&self, handle: &AssetId<OvermapBufferAsset>) -> Option<Overzone> {
        self.storage.region(handle)
    }
}
