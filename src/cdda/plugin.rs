use crate::prelude::{
    Map, MapLoader, MapMemory, MapMemoryLoader, Overmap, OvermapBuffer, OvermapLoader,
};
use bevy::prelude::*;

pub(crate) struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Map>();
        app.init_asset_loader::<MapLoader>();

        app.init_asset::<MapMemory>();
        app.init_asset_loader::<MapMemoryLoader>();

        app.init_asset::<Overmap>();
        app.init_asset_loader::<OvermapLoader<Overmap>>();

        app.init_asset::<OvermapBuffer>();
        app.init_asset_loader::<OvermapLoader<OvermapBuffer>>();
    }
}
