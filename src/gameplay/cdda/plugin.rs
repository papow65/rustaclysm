use crate::application::ApplicationState;
use crate::gameplay::cdda::systems::{create_cdda_resources, remove_cdda_resources};
use crate::gameplay::cdda::{Infos, MapLoader, MapMemoryLoader, OvermapLoader};
use crate::gameplay::{MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset, TileLoader};
use crate::util::AsyncResourcePlugin;
use bevy::prelude::{App, AssetApp, OnEnter, OnExit, Plugin};

pub(crate) struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AsyncResourcePlugin::<Infos>::default());
        app.add_plugins(AsyncResourcePlugin::<TileLoader>::default());

        app.init_asset::<MapAsset>();
        app.init_asset_loader::<MapLoader>();

        app.init_asset::<MapMemoryAsset>();
        app.init_asset_loader::<MapMemoryLoader>();

        app.init_asset::<OvermapAsset>();
        app.init_asset_loader::<OvermapLoader<OvermapAsset>>();

        app.init_asset::<OvermapBufferAsset>();
        app.init_asset_loader::<OvermapLoader<OvermapBufferAsset>>();

        app.add_systems(OnEnter(ApplicationState::Gameplay), create_cdda_resources);
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_cdda_resources);
    }
}
