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
    commands.remove_resource::<VisualizationUpdate>();
    commands.remove_resource::<Explored>();
    commands.remove_resource::<Sav>();
    commands.remove_resource::<Timeouts>();
    commands.remove_resource::<ZoneLevelIds>();
    commands.remove_resource::<CameraOffset>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<PlayerActionState>();
    commands.remove_resource::<StatusTextSections>();

    // RelativeSegments has to stay loaded, because it takes about 2 seconds to create and only contains static data.
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_gameplay(
    mut commands: Commands,
    root_entities: Query<
        Entity,
        Or<(
            With<ZoneLevel>,
            With<SubzoneLevel>,
            With<ManualRoot>,
            With<Node>,
            With<Message>,
        )>,
    >,
) {
    for root_entity in root_entities.iter() {
        commands.entity(root_entity).despawn_recursive();
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn disable_screen_state(mut next_state: ResMut<NextState<GameplayScreenState>>) {
    next_state.set(GameplayScreenState::Inapplicable);
}
