use crate::gameplay::{cdda::asset_storage::AssetStorage, Overzone, ZoneLevel};
use bevy::prelude::Commands;
use cdda::{Map, MapMemory, Overmap, OvermapBuffer};

pub(super) fn create_cdda_resources(mut commands: Commands) {
    commands.insert_resource(AssetStorage::<Map, ZoneLevel>::default());
    commands.insert_resource(AssetStorage::<MapMemory, ZoneLevel>::default());
    commands.insert_resource(AssetStorage::<Overmap, Overzone>::default());
    commands.insert_resource(AssetStorage::<OvermapBuffer, Overzone>::default());
}

pub(super) fn remove_cdda_resources(mut commands: Commands) {
    commands.remove_resource::<AssetStorage<Map, ZoneLevel>>();
    commands.remove_resource::<AssetStorage<MapMemory, ZoneLevel>>();
    commands.remove_resource::<AssetStorage<Overmap, Overzone>>();
    commands.remove_resource::<AssetStorage<OvermapBuffer, Overzone>>();
}
