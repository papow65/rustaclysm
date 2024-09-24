use crate::gameplay::{
    AppearanceCache, CameraOffset, Expanded, Explored, GameplayCounter, InstructionQueue, Location,
    MeshCaches, SubzoneLevelEntities, Timeouts, VisualizationUpdate, ZoneLevelEntities,
    ZoneLevelIds,
};
use bevy::prelude::{Commands, ResMut};
use cdda_json_files::Sav;

pub(crate) fn remove_gameplay_resources(mut commands: Commands) {
    commands.remove_resource::<AppearanceCache>();
    commands.remove_resource::<CameraOffset>();
    commands.remove_resource::<Expanded>();
    commands.remove_resource::<Explored>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<Location>();
    commands.remove_resource::<MeshCaches>();
    commands.remove_resource::<Sav>();
    commands.remove_resource::<Timeouts>();
    commands.remove_resource::<SubzoneLevelEntities>();
    commands.remove_resource::<VisualizationUpdate>();
    commands.remove_resource::<ZoneLevelEntities>();
    commands.remove_resource::<ZoneLevelIds>();
    // We don't remove the event resources, because that breaks the event readers.
}

pub(crate) fn increase_counter(mut counter: ResMut<GameplayCounter>) {
    counter.0 += 1;
}
