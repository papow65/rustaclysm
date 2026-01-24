use crate::text::{DebugTextShown, Fonts, add_missing_font, add_missing_pickable};
use bevy::prelude::{App, Plugin, PostUpdate};

pub(crate) struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fonts>();
        app.init_resource::<DebugTextShown>();

        app.add_systems(PostUpdate, (add_missing_font, add_missing_pickable));
    }
}
