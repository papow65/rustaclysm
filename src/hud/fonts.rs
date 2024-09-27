use crate::common::AssetPaths;
use crate::hud::colors::SOFT_TEXT_COLOR;
use bevy::prelude::{AssetServer, Color, Font, Handle, Resource, TextStyle};

#[derive(Resource)]
pub(crate) struct Fonts {
    fira: Handle<Font>,
}

impl Fonts {
    const REGULAR_FONT_SIZE: f32 = 16.0;
    const LARGISH_FONT_SIZE: f32 = 22.0;
    const LARGE_FONT_SIZE: f32 = 40.0;
    const HUGE_FONT_SIZE: f32 = 120.0;

    pub(crate) fn new(asset_server: &AssetServer) -> Self {
        Self {
            fira: asset_server.load(AssetPaths::fonts().join("FiraMono-Medium.otf")),
        }
    }

    pub(crate) fn regular(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: Self::REGULAR_FONT_SIZE,
            color,
        }
    }

    pub(crate) fn standard(&self) -> TextStyle {
        self.regular(SOFT_TEXT_COLOR)
    }

    pub(crate) fn largish(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: Self::LARGISH_FONT_SIZE,
            color,
        }
    }

    pub(crate) fn large(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: Self::LARGE_FONT_SIZE,
            color,
        }
    }

    pub(crate) fn huge(&self, color: Color) -> TextStyle {
        TextStyle {
            font: self.fira.clone(),
            font_size: Self::HUGE_FONT_SIZE,
            color,
        }
    }
}
