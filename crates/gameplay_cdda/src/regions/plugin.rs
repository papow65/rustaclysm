use crate::regions::{AssetStorage, Exploration, MapLoader, MapMemoryLoader, OvermapLoader};
use crate::{MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset};
use application_state::ApplicationState;
use bevy::prelude::{App, AssetApp as _, Plugin, StateScopedMessagesAppExt as _};
use gameplay_resource::GameplayResourcePlugin;

pub(crate) struct RegionsPlugin;

impl Plugin for RegionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Exploration>()
            .clear_messages_on_exit::<Exploration>(ApplicationState::Gameplay);

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
