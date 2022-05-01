use bevy::prelude::*;

use super::super::components::{Pos, SIZE};
use super::super::resources::Spawner;

#[allow(clippy::needless_pass_by_value)]
pub fn maximize_window(mut windows: ResMut<Windows>) {
    windows.primary_mut().set_maximized(true);
}

#[allow(clippy::needless_pass_by_value)]
pub fn add_entities(
    commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let mut spawner = Spawner::new(commands, &mut materials, &mut meshes, &asset_server);
    spawner.spawn_gui();
    //spawner.spawn_grid_lines();
    spawner.spawn_floors();
    spawner.spawn_house();
    spawner.spawn_characters();
    spawner.spawn_containables();
    spawner.spawn_window_wall();

    spawner.load_cdda_region(
        Pos(100, 0, 214),
        Pos(SIZE.0 / 24, SIZE.1, SIZE.2 / 24 - 2),
        Pos(0, 0, 48),
    );
    spawner.load_cdda_region(
        Pos(102, 0, 212),
        Pos(SIZE.0 / 24 - 2, SIZE.1, 2),
        Pos(48, 0, 0),
    );
}
