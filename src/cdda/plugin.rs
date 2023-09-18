use crate::prelude::{Map, MapLoader, Overmap, OvermapBuffer, OvermapLoader};
use bevy::prelude::*;

pub(crate) struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Map>();
        app.init_asset_loader::<MapLoader>();

        app.add_asset::<OvermapBuffer>();
        app.add_asset::<Overmap>();
        app.init_asset_loader::<OvermapLoader>();
    }
}
