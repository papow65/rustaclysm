use crate::gameplay::cdda::{asset_storage::AssetStorage, paths::OvermapBufferPath};
use crate::gameplay::{ActiveSav, AssetState, Overzone};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};
use cdda::OvermapBuffer;

#[derive(SystemParam)]
pub(crate) struct OvermapBufferManager<'w> {
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<OvermapBuffer, Overzone>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<OvermapBuffer>>,
}

impl<'w> OvermapBufferManager<'w> {
    pub(crate) fn get(&mut self, overzone: Overzone) -> AssetState<OvermapBuffer> {
        let path = OvermapBufferPath::new(&self.active_sav.sav_path(), overzone);
        self.storage
            .handle(&self.asset_server, &self.assets, overzone, path)
    }

    pub(crate) fn overzone(&self, handle: &AssetId<OvermapBuffer>) -> Option<Overzone> {
        self.storage.region(handle)
    }
}
