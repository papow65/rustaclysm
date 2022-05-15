use crate::prelude::*;
use bevy::prelude::*;

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

    let offset = Zone { x: 100, z: 212 }.zone_level(0).base_pos();
    spawner.spawn_floors(offset);
    spawner.spawn_house(offset);
    spawner.spawn_characters(offset);
    spawner.spawn_containables(offset);
    spawner.spawn_window_wall(offset);
}
