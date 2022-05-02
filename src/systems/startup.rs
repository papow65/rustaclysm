use bevy::prelude::*;

use super::super::components::Zone;
use super::super::resources::{Spawner, SpawnerData};

#[allow(clippy::needless_pass_by_value)]
pub fn maximize_window(mut windows: ResMut<Windows>) {
    windows.primary_mut().set_maximized(true);
}

#[allow(clippy::needless_pass_by_value)]
pub fn create_spawner_data(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(SpawnerData::new(&mut materials, &mut meshes, &asset_server));
}

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_initial_entities(mut spawner: Spawner) {
    spawner.spawn_gui();
    spawner.spawn_floors();
    spawner.spawn_house();
    spawner.spawn_characters();
    spawner.spawn_containables();
    spawner.spawn_window_wall();

    spawner.load_cdda_region(Zone { x: 0, z: 0 }, 2);
}
