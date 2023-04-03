use crate::prelude::*;
use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn remove_gameplay_resources(mut commands: Commands) {
    commands.remove_resource::<Infos>();
    commands.remove_resource::<Location>();
    commands.remove_resource::<SubzoneLevelEntities>();
    commands.remove_resource::<ZoneLevelEntities>();
    commands.remove_resource::<InstructionQueue>();
    commands.remove_resource::<TileCaches>();
    commands.remove_resource::<VisualizationUpdate>();
    commands.remove_resource::<CustomData>();
    commands.remove_resource::<Explored>();
    commands.remove_resource::<Sav>();
    commands.remove_resource::<Timeouts>();
    commands.remove_resource::<CustomData>();
    commands.remove_resource::<ZoneLevelIds>();
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
        )>,
    >,
) {
    for root_entity in root_entities.iter() {
        commands.entity(root_entity).despawn_recursive();
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn free_assets(asset_server: Res<AssetServer>) {
    asset_server.free_unused_assets();
}
