use crate::gameplay::{
    ActiveSav, AppearanceCache, CameraOffset, Expanded, Explored, Infos, InstructionQueue, Level,
    Location, MeshCaches, Spawner, SubzoneLevelEntities, Timeouts, VisualizationUpdate, Zone,
    ZoneLevelEntities, ZoneLevelIds,
};
use bevy::prelude::{Commands, Res};
use units::Timestamp;

/// Create resources that do not need other resources
pub(crate) fn create_independent_resources(mut commands: Commands) {
    // Not persisted between gameplays
    commands.insert_resource(AppearanceCache::default());
    commands.insert_resource(CameraOffset::default());
    commands.insert_resource(Expanded::default());
    commands.insert_resource(Explored::default());
    commands.insert_resource(InstructionQueue::default());
    commands.insert_resource(Location::default());
    commands.insert_resource(MeshCaches::default());
    commands.insert_resource(SubzoneLevelEntities::default());
    commands.insert_resource(VisualizationUpdate::Smart);
    commands.insert_resource(ZoneLevelEntities::default());
    commands.insert_resource(ZoneLevelIds::default());
}

/// Create resources that need other resources
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn create_dependent_resources(mut commands: Commands, active_sav: Res<ActiveSav>) {
    let sav = active_sav.sav();
    let season_length = 91; // TODO load from worldoptions.json
    let timestamp = Timestamp::new(sav.turn, season_length);
    commands.insert_resource(Timeouts::new(timestamp));
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn spawn_initial_entities(
    infos: Res<Infos>,
    active_sav: Res<ActiveSav>,
    mut spawner: Spawner,
) {
    spawner.spawn_light();

    let sav = active_sav.sav();
    let spawn_pos = Zone {
        x: i32::from(sav.om_x) * 180 + i32::from(sav.levx) / 2,
        z: i32::from(sav.om_y) * 180 + i32::from(sav.levy) / 2,
    }
    .zone_level(Level::new(sav.levz))
    .base_corner()
    .horizontal_offset(
        12 * i32::from(sav.levx % 2) + 24,
        12 * i32::from(sav.levy % 2) + 24,
    );
    spawner.spawn_characters(&infos, spawn_pos);
}
