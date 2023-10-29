use crate::prelude::{Map, MapLoader, OvermapAsset, OvermapLoader};
use bevy::prelude::*;

pub(crate) struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Map>();
        app.init_asset_loader::<MapLoader>();

        app.init_asset::<OvermapAsset>();
        app.init_asset_loader::<OvermapLoader>();
    }
}
