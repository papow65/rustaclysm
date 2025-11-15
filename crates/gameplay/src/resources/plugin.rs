use crate::{
    CameraOffset, ElevationVisibility, Expanded, Explored, RelativeSegments, VisualizationUpdate,
    ZoneLevelIds,
};
use bevy::prelude::{App, Plugin};
use gameplay_resource::gameplay_resource_plugin;
use util::async_resource_plugin;

pub(crate) struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            async_resource_plugin::<RelativeSegments>,
            gameplay_resource_plugin::<CameraOffset>,
            gameplay_resource_plugin::<Expanded>,
            gameplay_resource_plugin::<Explored>,
            gameplay_resource_plugin::<VisualizationUpdate>,
            gameplay_resource_plugin::<ZoneLevelIds>,
        ));

        app.insert_resource(ElevationVisibility::default());
    }
}
