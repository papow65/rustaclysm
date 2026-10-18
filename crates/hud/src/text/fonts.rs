use bevy::prelude::{AssetServer, FontSize, FontSource, FromWorld, Resource, TextFont, World};
use util::AssetPaths;

#[derive(Resource)]
pub(super) struct FiraFont {
    pub(super) font_source: FontSource,
}

impl FromWorld for FiraFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world
            .get_resource::<AssetServer>()
            .expect("AssetServer should exist");
        let handle = asset_server.load(AssetPaths::fonts().join("FiraMono-Medium.otf"));
        Self {
            font_source: FontSource::Handle(handle),
        }
    }
}

const REGULAR_FONT_SIZE: FontSize = FontSize::Px(13.0);
const LARGISH_FONT_SIZE: FontSize = FontSize::Px(18.0);
const LARGE_FONT_SIZE: FontSize = FontSize::Px(33.0);
const HUGE_FONT_SIZE: FontSize = FontSize::Px(100.0);

#[must_use]
pub fn regular_font() -> TextFont {
    TextFont {
        font_size: REGULAR_FONT_SIZE,
        ..TextFont::default()
    }
}

#[must_use]
pub fn largish_font() -> TextFont {
    TextFont {
        font_size: LARGISH_FONT_SIZE,
        ..TextFont::default()
    }
}

#[must_use]
pub fn large_font() -> TextFont {
    TextFont {
        font_size: LARGE_FONT_SIZE,
        ..TextFont::default()
    }
}

#[must_use]
pub fn huge_font() -> TextFont {
    TextFont {
        font_size: HUGE_FONT_SIZE,
        ..TextFont::default()
    }
}
