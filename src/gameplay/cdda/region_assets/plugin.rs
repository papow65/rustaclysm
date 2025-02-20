use crate::gameplay::GameplayResourcePlugin;
use crate::gameplay::cdda::region_assets::{
    AssetStorage, MapLoader, MapMemoryLoader, OvermapLoader,
};
use crate::gameplay::cdda::{MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset};
use bevy::prelude::{App, AssetApp as _, Plugin};

pub(in super::super) struct RegionAssetsPlugin;

impl Plugin for RegionAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayResourcePlugin::<AssetStorage<MapAsset>>::default(),
            GameplayResourcePlugin::<AssetStorage<MapMemoryAsset>>::default(),
            GameplayResourcePlugin::<AssetStorage<OvermapAsset>>::default(),
            GameplayResourcePlugin::<AssetStorage<OvermapBufferAsset>>::default(),
        ));

        app.init_asset::<MapAsset>();
        app.init_asset_loader::<MapLoader>();

        app.init_asset::<MapMemoryAsset>();
        app.init_asset_loader::<MapMemoryLoader>();

        app.init_asset::<OvermapAsset>();
        app.init_asset_loader::<OvermapLoader<OvermapAsset>>();

        app.init_asset::<OvermapBufferAsset>();
        app.init_asset_loader::<OvermapLoader<OvermapBufferAsset>>();
    }
}
