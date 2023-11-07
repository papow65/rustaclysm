use crate::prelude::*;
use bevy::prelude::*;
use futures_lite::future::{block_on, poll_once};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn load_relative_segments(
    mut commands: Commands,
    mut relative_segments_generator: ResMut<RelativeSegmentsGenerator>,
) {
    if let Some(relative_segments) = block_on(poll_once(&mut relative_segments_generator.task)) {
        commands.insert_resource(relative_segments);
    }
}

/// Create resources that do not need other resources
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_independent_resources(mut commands: Commands) {
    // Not persisted between gameplays
    commands.insert_resource(Infos::new());
    commands.insert_resource(Location::default());
    commands.insert_resource(SubzoneLevelEntities::default());
    commands.insert_resource(ZoneLevelEntities::default());
    commands.insert_resource(Explored::default());
    commands.insert_resource(ZoneLevelIds::default());
    commands.insert_resource(TileLoader::new());
    commands.insert_resource(InstructionQueue::default());
    commands.insert_resource(AppearanceCache::default());
    commands.insert_resource(MeshCaches::default());
    commands.insert_resource(CameraOffset::default());
    commands.insert_resource(InstructionQueue::default());
    commands.insert_resource(PlayerActionState::default());
    commands.insert_resource(StatusTextSections::default());
    commands.insert_resource(VisualizationUpdate::Smart);
    commands.insert_resource(Events::<Message>::default());
    commands.insert_resource(Events::<SpawnSubzoneLevel>::default());
    commands.insert_resource(Events::<CollapseZoneLevel>::default());
    commands.insert_resource(Events::<SpawnZoneLevel>::default());
    commands.insert_resource(Events::<UpdateZoneLevelVisibility>::default());
    commands.insert_resource(Events::<DespawnZoneLevel>::default());
    commands.insert_resource(Events::<ActorEvent<Stay>>::default());
    commands.insert_resource(Events::<ActorEvent<Step>>::default());
    commands.insert_resource(Events::<ActorEvent<Attack>>::default());
    commands.insert_resource(Events::<ActorEvent<Smash>>::default());
    commands.insert_resource(Events::<ActorEvent<Close>>::default());
    commands.insert_resource(Events::<ActorEvent<Wield>>::default());
    commands.insert_resource(Events::<ActorEvent<Unwield>>::default());
    commands.insert_resource(Events::<ActorEvent<Pickup>>::default());
    commands.insert_resource(Events::<ActorEvent<MoveItem>>::default());
    commands.insert_resource(Events::<ActorEvent<ExamineItem>>::default());
    commands.insert_resource(Events::<ActorEvent<ChangePace>>::default());
    commands.insert_resource(Events::<ActorEvent<StaminaImpact>>::default());
    commands.insert_resource(Events::<ActorEvent<Timeout>>::default());
    commands.insert_resource(Events::<ActorEvent<Damage>>::default());
    commands.insert_resource(Events::<ActorEvent<Healing>>::default());
    commands.insert_resource(Events::<TerrainEvent<Damage>>::default());
    commands.insert_resource(Events::<TerrainEvent<Toggle>>::default());
}

/// Create resources that need other resources
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_dependent_resources(mut commands: Commands, paths: Res<Paths>) {
    let sav = Sav::try_from(&paths.sav_path()).expect("Loading sav file failed");
    let season_length = 91; // TODO load from worldoptions.json
    let timestamp = Timestamp::new(sav.turn, season_length);

    commands.insert_resource(sav);
    commands.insert_resource(OvermapBufferManager::new(paths.sav_path()));
    commands.insert_resource(OvermapManager::new(paths.world_path()));
    commands.insert_resource(MapManager::new(paths.world_path()));
    commands.insert_resource(Timeouts::new(timestamp));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_initial_entities(infos: Res<Infos>, sav: Res<Sav>, mut spawner: Spawner) {
    spawner.spawn_light();

    let offset = Zone {
        x: i32::from(sav.om_x) * 180 + i32::from(sav.levx) / 2,
        z: i32::from(sav.om_y) * 180 + i32::from(sav.levy) / 2,
    }
    .zone_level(Level::new(sav.levz))
    .base_pos()
    .offset(PosOffset {
        x: 12 * i32::from(sav.levx % 2),
        level: LevelOffset::ZERO,
        z: 12 * i32::from(sav.levy % 2),
    })
    .unwrap()
    .offset(PosOffset {
        x: 24,
        level: LevelOffset::ZERO,
        z: 24,
    })
    .unwrap()
        - Pos::ORIGIN;
    spawner.spawn_characters(&infos, offset);
}
