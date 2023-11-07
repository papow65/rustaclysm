use crate::prelude::*;
use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_gameplay_resources(mut commands: Commands) {
    commands.remove_resource::<Infos>();
    commands.remove_resource::<Location>();
    commands.remove_resource::<SubzoneLevelEntities>();
    commands.remove_resource::<ZoneLevelEntities>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<AppearanceCache>();
    commands.remove_resource::<MeshCaches>();
    commands.remove_resource::<MapManager>();
    commands.remove_resource::<VisualizationUpdate>();
    commands.remove_resource::<Explored>();
    commands.remove_resource::<Sav>();
    commands.remove_resource::<Timeouts>();
    commands.remove_resource::<OvermapBufferManager>();
    commands.remove_resource::<OvermapManager>();
    commands.remove_resource::<MapManager>();
    commands.remove_resource::<ZoneLevelIds>();
    commands.remove_resource::<CameraOffset>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<PlayerActionState>();
    commands.remove_resource::<StatusTextSections>();
    // We don't remove the event resources, because that breaks the event readers.
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn clear_gameplay_events<T: Event>(mut events: ResMut<Events<T>>) {
    events.clear();
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn disable_screen_state(mut next_state: ResMut<NextState<GameplayScreenState>>) {
    next_state.set(GameplayScreenState::Inapplicable);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn increase_counter(mut counter: ResMut<GameplayCounter>) {
    counter.0 += 1;
}
