use crate::prelude::Paths;
use bevy::prelude::{AssetServer, Font, Handle};

pub(crate) fn default_font(asset_server: &AssetServer) -> Handle<Font> {
    asset_server.load(Paths::fonts_path().join("FiraMono-Medium.otf"))
}
