use bevy::prelude::{AssetServer, Font, FromWorld, Handle, Resource, TextFont, World};
use util::AssetPaths;

#[derive(Resource)]
pub struct Fonts {
    fira: Handle<Font>,
}

impl Fonts {
    const REGULAR_FONT_SIZE: f32 = 13.0;
    const LARGISH_FONT_SIZE: f32 = 18.0;
    const LARGE_FONT_SIZE: f32 = 33.0;
    const HUGE_FONT_SIZE: f32 = 100.0;

    #[must_use]
    pub fn new(asset_server: &AssetServer) -> Self {
        Self {
            fira: asset_server.load(AssetPaths::fonts().join("FiraMono-Medium.otf")),
        }
    }

    #[must_use]
    pub fn regular(&self) -> TextFont {
        TextFont {
            font: self.fira.clone(),
            font_size: Self::REGULAR_FONT_SIZE,
            ..TextFont::default()
        }
    }

    #[must_use]
    pub fn largish(&self) -> TextFont {
        TextFont {
            font: self.fira.clone(),
            font_size: Self::LARGISH_FONT_SIZE,
            ..TextFont::default()
        }
    }

    #[must_use]
    pub fn large(&self) -> TextFont {
        TextFont {
            font: self.fira.clone(),
            font_size: Self::LARGE_FONT_SIZE,
            ..TextFont::default()
        }
    }

    #[must_use]
    pub fn huge(&self) -> TextFont {
        TextFont {
            font: self.fira.clone(),
            font_size: Self::HUGE_FONT_SIZE,
            ..TextFont::default()
        }
    }
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource().expect("AssetServer should exist");
        Self::new(asset_server)
    }
}
