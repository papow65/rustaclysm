use crate::common::Paths;
use crate::gameplay::cdda::{asset_storage::AssetStorage, paths::MapMemoryPath};
use crate::gameplay::{AssetState, SubzoneLevel, Zone, ZoneLevel};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};
use cdda::{MapMemory, SubmapMemory};

#[derive(SystemParam)]
pub(crate) struct MapMemoryManager<'w> {
    paths: Res<'w, Paths>,
    storage: ResMut<'w, AssetStorage<MapMemory, ZoneLevel>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<MapMemory>>,
}

impl<'w> MapMemoryManager<'w> {
    pub(crate) fn submap(&mut self, subzone_level: SubzoneLevel) -> AssetState<SubmapMemory> {
        let sav_path = self.paths.sav_path();
        let zone = ZoneLevel::from(subzone_level).zone;
        let base_zone_level = ZoneLevel {
            zone: Zone {
                x: 4 * zone.x.div_euclid(4),
                z: 4 * zone.z.div_euclid(4),
            },
            level: subzone_level.level,
        };
        let path = MapMemoryPath::new(&sav_path, base_zone_level);
        //println!("{:?}", &path);
        let map_memory =
            self.storage
                .handle(&self.asset_server, &self.assets, base_zone_level, path);
        match map_memory {
            AssetState::Available { asset: map_memory } => {
                let index = subzone_level.z.rem_euclid(8) as usize * 8
                    + subzone_level.x.rem_euclid(8) as usize;
                AssetState::Available {
                    asset: &map_memory.0[index],
                }
            }
            AssetState::Loading => AssetState::Loading,
            AssetState::Nonexistent => AssetState::Nonexistent,
        }
    }

    pub(crate) fn base_zone_level(&self, handle: &AssetId<MapMemory>) -> Option<ZoneLevel> {
        self.storage.region(handle).inspect(|zone_level| {
            assert!(
                zone_level.zone.x.rem_euclid(4) == 0 && zone_level.zone.z.rem_euclid(4) == 0,
                "Misaligned zone level: {zone_level:?}"
            );
        })
    }
}
