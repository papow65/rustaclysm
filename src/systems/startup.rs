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
    let sav = Sav::try_from(&paths.sav_path()).expect("Loading sav file failed");
    let timouts = Timeouts::new(sav.turn);

    commands.insert_resource(CustomData::new(&mut materials, &mut meshes, &asset_server));
    commands.insert_resource(Explored::new(paths.sav_path()));
    commands.insert_resource(sav);
    commands.insert_resource(TileLoader::new(&asset_server));
    commands.insert_resource(timouts);
    commands.insert_resource(ZoneLevelNames::new(paths.world_path()));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_initial_entities(sav: Res<Sav>, mut spawner: Spawner) {
    spawner.spawn_light();

    let offset = Zone {
        x: i32::from(sav.om_x) * 180 + i32::from(sav.levx) / 2,
        z: i32::from(sav.om_y) * 180 + i32::from(sav.levy) / 2,
    }
    .zone_level(Level::new(sav.levz))
    .base_pos()
    .offset(PosOffset {
        x: 12 * (i32::from(sav.levx) % 2),
        level: LevelOffset::ZERO,
        z: 12 * (i32::from(sav.levy) % 2),
    })
    .unwrap()
    .offset(PosOffset {
        x: 24,
        level: LevelOffset::ZERO,
        z: 24,
    }) // experimental
    .unwrap()
        - Pos::ORIGIN;
    spawner.spawn_floors(offset);
    spawner.spawn_house(offset);
    spawner.spawn_characters(offset);
    spawner.spawn_containables(offset);
    spawner.spawn_window_wall(offset);
}
