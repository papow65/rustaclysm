use crate::prelude::*;
use bevy::prelude::*;

/// Create resources that do not need other resources and should not persist between two gameplays
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_independent_resources(mut commands: Commands) {
    commands.insert_resource(Infos::new());
    commands.insert_resource(Location::default());
    commands.insert_resource(SubzoneLevelEntities::default());
    commands.insert_resource(ZoneLevelEntities::default());
    commands.insert_resource(InstructionQueue::default());
    commands.insert_resource(TileCaches::default());
    commands.insert_resource(Maps::default());
    commands.insert_resource(CameraOffset::default());
    commands.insert_resource(PlayerActionState::default());
    commands.insert_resource(VisualizationUpdate::Smart);
}

/// Create resources that need other resources
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_dependent_resources(mut commands: Commands, paths: Res<Paths>) {
    let sav = Sav::try_from(&paths.sav_path()).expect("Loading sav file failed");
    let turn = sav.turn;

    commands.insert_resource(Explored::new(paths.sav_path()));
    commands.insert_resource(sav);
    commands.insert_resource(TileLoader::new());
    commands.insert_resource(Timeouts::new(turn));
    commands.insert_resource(ZoneLevelIds::new(paths.world_path()));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_initial_entities(mut commands: Commands, sav: Res<Sav>, mut spawner: Spawner) {
    let root = commands
        .spawn(SpatialBundle::default())
        .insert(ManualRoot)
        .id();

    spawner.spawn_light(root);

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
    })
    .unwrap()
        - Pos::ORIGIN;
    spawner.spawn_characters(root, offset);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn finish_loading(mut next_state: ResMut<NextState<GameplayScreenState>>) {
    next_state.set(GameplayScreenState::Base);
}
