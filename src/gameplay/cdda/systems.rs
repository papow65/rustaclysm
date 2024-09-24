use crate::gameplay::cdda::asset_storage::AssetStorage;
use crate::gameplay::{
    ActiveSav, MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset, Overzone, ZoneLevel,
};
use bevy::prelude::Commands;

pub(super) fn create_cdda_resources(mut commands: Commands) {
    commands.insert_resource(AssetStorage::<MapAsset, ZoneLevel>::default());
    commands.insert_resource(AssetStorage::<MapMemoryAsset, ZoneLevel>::default());
    commands.insert_resource(AssetStorage::<OvermapAsset, Overzone>::default());
    commands.insert_resource(AssetStorage::<OvermapBufferAsset, Overzone>::default());

    // ActiveSav is created in the main menu
}

pub(super) fn remove_cdda_resources(mut commands: Commands) {
    commands.remove_resource::<AssetStorage<MapAsset, ZoneLevel>>();
    commands.remove_resource::<AssetStorage<MapMemoryAsset, ZoneLevel>>();
    commands.remove_resource::<AssetStorage<OvermapAsset, Overzone>>();
    commands.remove_resource::<AssetStorage<OvermapBufferAsset, Overzone>>();

    commands.remove_resource::<ActiveSav>();
}
