use crate::gameplay::{
    CameraOffset, ElevationVisibility, Expanded, Explored, GameplayResourcePlugin,
    InstructionQueue, Location, RelativeSegments, SubzoneLevelEntities, TileLoader,
    VisualizationUpdate, ZoneLevelEntities, ZoneLevelIds,
};
use crate::util::{load_async_resource, AsyncResourceLoader};
use bevy::prelude::{App, Plugin, Update};

pub(crate) struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
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

        // These resources persist between gameplays.
        app.insert_resource(ElevationVisibility::default())
            .insert_resource(AsyncResourceLoader::<RelativeSegments>::default())
            .insert_resource(AsyncResourceLoader::<TileLoader>::default());

        app.add_systems(
            Update,
            (
                // Resources that take a while to load, are loaded in the background, independent of the current ApplicationState
                load_async_resource::<RelativeSegments>(),
                load_async_resource::<TileLoader>(),
            ),
        );
    }
}
