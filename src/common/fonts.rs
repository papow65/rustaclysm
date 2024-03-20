use crate::prelude::Paths;
use bevy::prelude::{AssetServer, Font, Handle, Resource};

#[derive(Default, Resource)]
pub(crate) struct Fonts {
    fira: Handle<Font>,
}

impl Fonts {
    pub(crate) fn new(asset_server: &AssetServer) -> Self {
        Self {
            fira: asset_server.load(Paths::fonts_path().join("FiraMono-Medium.otf")),
        }
    }

    pub(crate) fn default(&self) -> Handle<Font> {
        self.fira.clone()
    }
}
