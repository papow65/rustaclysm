use crate::common::{PathFor, Paths};
use crate::gameplay::cdda::{asset_storage::AssetStorage, paths::MapPath};
use crate::gameplay::{AssetState, SubzoneLevel, ZoneLevel};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};
use cdda::{Map, Submap};

#[derive(SystemParam)]
pub(crate) struct MapManager<'w> {
    paths: Res<'w, Paths>,
    storage: ResMut<'w, AssetStorage<Map, ZoneLevel>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<Map>>,
}

impl<'w> MapManager<'w> {
    fn path(&self, zone_level: ZoneLevel) -> PathFor<Map> {
        let world_map = self.paths.world_path();
        MapPath::new(&world_map, zone_level)
    }

    fn map(&mut self, zone_level: ZoneLevel) -> AssetState<Map> {
        let path = self.path(zone_level);
        self.storage
            .handle(&self.asset_server, &self.assets, zone_level, path)
    }

    pub(crate) fn submap(&mut self, subzone_level: SubzoneLevel) -> AssetState<Submap> {
        let zone_level = ZoneLevel::from(subzone_level);
        match self.map(zone_level) {
            AssetState::Available { asset: map } => AssetState::Available {
                asset: &map.0[subzone_level.index()],
            },
            AssetState::Loading => AssetState::Loading,
            AssetState::Nonexistent => AssetState::Nonexistent,
        }
    }

    pub(crate) fn zone_level(&self, handle: &AssetId<Map>) -> Option<ZoneLevel> {
        self.storage.region(handle)
    }
}
