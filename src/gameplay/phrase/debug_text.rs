use bevy::prelude::{Component, Resource, TextFont};

#[derive(Clone, Copy, Debug, Component)]
#[component(immutable)]
pub(crate) struct DebugText;

#[derive(Clone, Copy, Debug, Default, Resource)]
pub(crate) struct DebugTextShown(pub(crate) bool);

impl DebugTextShown {
    pub(crate) const fn text_font(self, mut text_font: TextFont) -> TextFont {
        if !self.0 {
            text_font.font_size = 0.0;
        }
        text_font
    }
}
