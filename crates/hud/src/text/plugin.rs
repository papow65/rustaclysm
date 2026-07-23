use crate::text::{
    DebugTextShown, FiraFont, add_missing_pickable, set_default_font_size, setup_global_font,
};
use bevy::prelude::{
    Added, App, AssetEvent, Font, IntoScheduleConfigs as _, Or, Plugin, PostUpdate, Text, TextSpan,
    any_match_filter, on_message,
};

pub(crate) struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FiraFont>();
        app.init_resource::<DebugTextShown>();

        app.add_systems(
            PostUpdate,
            (
                setup_global_font.run_if(on_message::<AssetEvent<Font>>),
                (set_default_font_size, add_missing_pickable)
                    .run_if(any_match_filter::<Or<(Added<Text>, Added<TextSpan>)>>),
            ),
        );
    }
}
