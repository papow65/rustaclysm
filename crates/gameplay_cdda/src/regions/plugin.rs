use crate::regions::{AssetStorage, Exploration, MapLoader, MapMemoryLoader, OvermapLoader};
use crate::{MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset};
use application_state::ApplicationState;
use bevy::prelude::{App, AssetApp as _, Plugin, StateScopedMessagesAppExt as _};
use gameplay_resource::gameplay_resource_plugin;

pub(crate) struct RegionsPlugin;

impl Plugin for RegionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Exploration>()
            .clear_messages_on_exit::<Exploration>(ApplicationState::Gameplay);

        app.add_plugins((
            gameplay_resource_plugin::<AssetStorage<MapAsset>>,
            gameplay_resource_plugin::<AssetStorage<MapMemoryAsset>>,
            gameplay_resource_plugin::<AssetStorage<OvermapAsset>>,
            gameplay_resource_plugin::<AssetStorage<OvermapBufferAsset>>,
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
