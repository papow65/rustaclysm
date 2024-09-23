use crate::application::ApplicationState;
use crate::gameplay::cdda::systems::{create_cdda_resources, remove_cdda_resources};
use crate::gameplay::cdda::{MapLoader, MapMemoryLoader, OvermapLoader};
use bevy::prelude::{App, AssetApp, OnEnter, OnExit, Plugin};
use cdda::{Map, MapMemory, Overmap, OvermapBuffer};

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

        app.add_systems(OnEnter(ApplicationState::Gameplay), create_cdda_resources);
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_cdda_resources);
    }
}
