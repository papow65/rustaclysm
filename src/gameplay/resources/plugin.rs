use crate::gameplay::{
    CameraOffset, ElevationVisibility, Expanded, Explored, GameplayResourcePlugin,
    InstructionQueue, Location, RelativeSegments, SubzoneLevelEntities, VisualizationUpdate,
    ZoneLevelEntities, ZoneLevelIds,
};
use bevy::prelude::{App, Plugin};
use util::AsyncResourcePlugin;

pub(crate) struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AsyncResourcePlugin::<RelativeSegments>::default(),
            GameplayResourcePlugin::<CameraOffset>::default(),
            GameplayResourcePlugin::<Expanded>::default(),
            GameplayResourcePlugin::<Explored>::default(),
            GameplayResourcePlugin::<InstructionQueue>::default(),
            GameplayResourcePlugin::<Location>::default(),
            GameplayResourcePlugin::<SubzoneLevelEntities>::default(),
            GameplayResourcePlugin::<VisualizationUpdate>::default(),
            GameplayResourcePlugin::<ZoneLevelEntities>::default(),
            GameplayResourcePlugin::<ZoneLevelIds>::default(),
        ));

        app.insert_resource(ElevationVisibility::default());
    }
}
