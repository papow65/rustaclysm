use crate::cdda::{Map, MapMemory, Overmap, OvermapBuffer, Sav};
use crate::gameplay::{
    AppearanceCache, AssetStorage, CameraOffset, Expanded, Explored, GameplayCounter,
    InstructionQueue, Location, MeshCaches, Overzone, SubzoneLevelEntities, Timeouts,
    VisualizationUpdate, ZoneLevel, ZoneLevelEntities, ZoneLevelIds,
};
use bevy::prelude::{Commands, Event, Events, ResMut};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_gameplay_resources(mut commands: Commands) {
    commands.remove_resource::<Location>();
    commands.remove_resource::<SubzoneLevelEntities>();
    commands.remove_resource::<ZoneLevelEntities>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<AppearanceCache>();
    commands.remove_resource::<MeshCaches>();
    commands.remove_resource::<VisualizationUpdate>();
    commands.remove_resource::<Expanded>();
    commands.remove_resource::<Explored>();
    commands.remove_resource::<Sav>();
    commands.remove_resource::<Timeouts>();
    commands.remove_resource::<AssetStorage<Overmap, Overzone>>();
    commands.remove_resource::<AssetStorage<OvermapBuffer, Overzone>>();
    commands.remove_resource::<AssetStorage<Map, ZoneLevel>>();
    commands.remove_resource::<AssetStorage<MapMemory, ZoneLevel>>();
    commands.remove_resource::<ZoneLevelIds>();
    commands.remove_resource::<CameraOffset>();
    commands.remove_resource::<InstructionQueue>();
    // We don't remove the event resources, because that breaks the event readers.
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn clear_gameplay_events<T: Event>(mut events: ResMut<Events<T>>) {
    events.clear();
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn increase_counter(mut counter: ResMut<GameplayCounter>) {
    counter.0 += 1;
}
