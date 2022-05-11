use bevy::prelude::*;

use crate::cdda::tile_loader::TileLoader;
use crate::resources::{CustomData, Spawner};

#[allow(clippy::needless_pass_by_value)]
pub fn maximize_window(mut windows: ResMut<Windows>) {
    windows.primary_mut().set_maximized(true);
}

#[allow(clippy::needless_pass_by_value)]
pub fn create_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TileLoader::new(&asset_server));
}

#[allow(clippy::needless_pass_by_value)]
pub fn create_custom_data(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(CustomData::new(&mut materials, &mut meshes, &asset_server));
}

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_initial_entities(mut spawner: Spawner) {
    spawner.spawn_light();
    spawner.spawn_floors();
    spawner.spawn_house();
    spawner.spawn_characters();
    spawner.spawn_containables();
    spawner.spawn_window_wall();
}
