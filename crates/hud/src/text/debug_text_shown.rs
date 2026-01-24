use bevy::prelude::{Resource, TextFont};

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct DebugTextShown(pub(super) bool);

impl DebugTextShown {
    pub(super) const fn text_font(self, mut text_font: TextFont) -> TextFont {
        if !self.0 {
            text_font.font_size = 0.0;
        }
        text_font
    }
}
