use crate::prelude::{
    Paths, HUGE_FONT_SIZE, LARGE_FONT_SIZE, LARGISH_FONT_SIZE, REGULAR_FONT_SIZE,
};
use bevy::prelude::{AssetServer, Color, Font, Handle, Resource, TextStyle};

#[derive(Resource)]
pub(crate) struct Fonts {
    fira: Handle<Font>,
}

impl Fonts {
    pub(crate) fn new(asset_server: &AssetServer) -> Self {
        Self {
            fira: asset_server.load(Paths::fonts_path().join("FiraMono-Medium.otf")),
        }
    }

    pub(crate) fn regular(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: REGULAR_FONT_SIZE,
            color,
        }
    }

    pub(crate) fn largish(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: LARGISH_FONT_SIZE,
            color,
        }
    }

    pub(crate) fn large(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: LARGE_FONT_SIZE,
            color,
        }
    }

    pub(crate) fn huge(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: HUGE_FONT_SIZE,
            color,
        }
    }
}
