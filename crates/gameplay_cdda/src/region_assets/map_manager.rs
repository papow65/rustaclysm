use crate::{ActiveSavExt as _, AssetState, MapAsset, MapPath, region_assets::AssetStorage};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetEvent, AssetServer, Assets, EventReader, Res, ResMut, warn};
use cdda_json_files::Submap;
use gameplay_cdda_active_sav::ActiveSav;
use gameplay_location::{SubzoneLevel, ZoneLevel};

#[derive(SystemParam)]
pub struct MapManager<'w, 's> {
    asset_events: EventReader<'w, 's, AssetEvent<MapAsset>>,
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<MapAsset>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<MapAsset>>,
}

impl MapManager<'_, '_> {
    fn path(&self, zone_level: ZoneLevel) -> MapPath {
        let world_map = self.active_sav.world_path();
        MapPath::new(&world_map, zone_level)
    }

    fn map(&mut self, zone_level: ZoneLevel) -> AssetState<'_, MapAsset> {
        let path = self.path(zone_level);
        self.storage
            .handle(&self.asset_server, &self.assets, zone_level, path)
    }

    pub fn submap(&mut self, subzone_level: SubzoneLevel) -> AssetState<'_, Submap> {
        let zone_level = ZoneLevel::from(subzone_level);
        match self.map(zone_level) {
            AssetState::Available { asset: map } => AssetState::Available {
                asset: &map.0.0[subzone_level.index()],
            },
            AssetState::Loading => AssetState::Loading,
            AssetState::Nonexistent => AssetState::Nonexistent,
        }
    }

    pub fn read_loaded_assets(&mut self) -> impl Iterator<Item = ZoneLevel> + use<'_> {
        self.asset_events.read().filter_map(|event| {
            if let AssetEvent::LoadedWithDependencies { id } = event {
                if let Some(zone_level) = self.storage.region(id) {
                    Some(zone_level)
                } else {
                    // This may be an asset of a previous gameplay.
                    warn!("Unknown map asset {id:?} loaded");
                    None
                }
            } else {
                None
            }
        })
    }
}
