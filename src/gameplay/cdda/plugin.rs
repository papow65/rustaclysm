use crate::application::ApplicationState;
use crate::gameplay::cdda::systems::{create_cdda_resources, remove_cdda_resources};
use crate::gameplay::cdda::{Infos, MapLoader, MapMemoryLoader, OvermapLoader};
use crate::gameplay::{MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset};
use crate::util::{load_async_resource, AsyncResourceLoader};
use bevy::prelude::{App, AssetApp, OnEnter, OnExit, Plugin, Update};

pub(crate) struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapAsset>();
        app.init_asset_loader::<MapLoader>();

        app.init_asset::<MapMemoryAsset>();
        app.init_asset_loader::<MapMemoryLoader>();

        app.init_asset::<OvermapAsset>();
        app.init_asset_loader::<OvermapLoader<OvermapAsset>>();

        app.init_asset::<OvermapBufferAsset>();
        app.init_asset_loader::<OvermapLoader<OvermapBufferAsset>>();

        app.insert_resource(AsyncResourceLoader::<Infos>::default());

        app.add_systems(OnEnter(ApplicationState::Gameplay), create_cdda_resources);
        app.add_systems(Update, load_async_resource::<Infos>());
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_cdda_resources);
    }
}
