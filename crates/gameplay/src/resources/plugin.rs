use crate::{
    CameraOffset, ElevationVisibility, Expanded, Explored, InstructionQueue, RelativeSegments,
    SubzoneLevelEntities, VisualizationUpdate, ZoneLevelIds,
};
use bevy::prelude::{App, Plugin};
use gameplay_location::{LocationCache, ZoneLevelCache};
use gameplay_resource::GameplayResourcePlugin;
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
            GameplayResourcePlugin::<LocationCache>::default(),
            GameplayResourcePlugin::<SubzoneLevelEntities>::default(),
            GameplayResourcePlugin::<VisualizationUpdate>::default(),
            GameplayResourcePlugin::<ZoneLevelCache>::default(),
            GameplayResourcePlugin::<ZoneLevelIds>::default(),
        ));

        app.insert_resource(ElevationVisibility::default());
    }
}
