use crate::{DespawnSubzoneLevel, DespawnZoneLevel};
use application_state::ApplicationState;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::{ScheduleSystem, SystemState};
use bevy::prelude::{
    Commands, EventReader, IntoScheduleConfigs as _, ResMut, World, debug, in_state, on_event,
};
use gameplay_location::SubzoneLevelCache;
use std::time::Instant;
use util::log_if_slow;

/// This should run last, to prevent Bevy crashing on despawned entities being modified.
pub(crate) fn despawn_systems() -> ScheduleConfigs<ScheduleSystem> {
    (
        despawn_subzone_levels.run_if(on_event::<DespawnSubzoneLevel>),
        despawn_zone_level.run_if(on_event::<DespawnZoneLevel>),
    )
        .run_if(in_state(ApplicationState::Gameplay))
}

/// This is an intentionally exclusive system to prevent an occasional panic.
/// See <https://bevyengine.org/learn/errors/b0003/>
fn despawn_subzone_levels(
    world: &mut World,
    sytem_state: &mut SystemState<(
        Commands,
        EventReader<DespawnSubzoneLevel>,
        ResMut<SubzoneLevelCache>,
    )>,
) {
    let start = Instant::now();

    let (mut commands, mut despawn_subzone_level_reader, subzone_level_cache) =
        sytem_state.get_mut(world);

    let despawned_count = despawn_subzone_level_reader.len();
    debug!("Despawning {despawned_count} subzone levels");

    for despawn_event in despawn_subzone_level_reader.read().take(64) {
        if let Some(entity) = subzone_level_cache.get(despawn_event.subzone_level) {
            commands.entity(entity).despawn();
        }
    }

    sytem_state.apply(world);

    log_if_slow("despawn_subzone_levels", start);
}

fn despawn_zone_level(
    mut commands: Commands,
    mut despawn_zone_level_reader: EventReader<DespawnZoneLevel>,
) {
    let start = Instant::now();

    debug!("Despawning {} zone levels", despawn_zone_level_reader.len());

    for despawn_zone_level_event in despawn_zone_level_reader.read() {
        let entity = despawn_zone_level_event.entity;
        commands.entity(entity).despawn();
    }

    log_if_slow("despawn_zone_level", start);
}
