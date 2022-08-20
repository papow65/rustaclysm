use crate::prelude::*;
use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn maximize_window(mut windows: ResMut<Windows>) {
    windows.primary_mut().set_maximized(true);
}

/// Create resources that need other resources
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_secondairy_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    paths: Res<Paths>,
) {
    commands.insert_resource(CustomData::new(&mut materials, &mut meshes, &asset_server));
    commands.insert_resource(Memory::new(paths.sav_path()));
    commands.insert_resource(Sav::try_from(&paths.sav_path()).expect("Loading sav file failed"));
    commands.insert_resource(TileLoader::new(&asset_server));
    commands.insert_resource(ZoneLevelNames::new(paths.world_path()));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_initial_entities(_sav: Res<Sav>, mut spawner: Spawner) {
    spawner.spawn_light();

    let offset = Zone { x: 12, z: 265 }.zone_level(Level::ZERO).base_pos();
    spawner.spawn_floors(offset);
    spawner.spawn_house(offset);
    spawner.spawn_characters(offset);
    spawner.spawn_containables(offset);
    spawner.spawn_window_wall(offset);
}
