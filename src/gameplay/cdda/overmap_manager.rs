use crate::gameplay::cdda::{asset_storage::AssetStorage, paths::OvermapPath};
use crate::gameplay::{ActiveSav, AssetState, Overzone};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};
use cdda_json_files::Overmap;

#[derive(SystemParam)]
pub(crate) struct OvermapManager<'w> {
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<Overmap, Overzone>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<Overmap>>,
}

impl<'w> OvermapManager<'w> {
    pub(crate) fn get(&mut self, overzone: Overzone) -> AssetState<Overmap> {
        let path = OvermapPath::new(&self.active_sav.world_path(), overzone);
        self.storage
            .handle(&self.asset_server, &self.assets, overzone, path)
    }

    pub(crate) fn overzone(&self, handle: &AssetId<Overmap>) -> Option<Overzone> {
        self.storage.region(handle)
    }
}
