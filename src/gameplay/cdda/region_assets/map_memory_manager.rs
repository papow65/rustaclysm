use crate::gameplay::cdda::paths::MapMemoryPath;
use crate::gameplay::cdda::region_assets::AssetStorage;
use crate::gameplay::{ActiveSav, AssetState, MapMemoryAsset, SubzoneLevel, Zone, ZoneLevel};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetId, AssetServer, Assets, Res, ResMut};
use cdda_json_files::SubmapMemory;

#[derive(SystemParam)]
pub(crate) struct MapMemoryManager<'w> {
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<MapMemoryAsset>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<MapMemoryAsset>>,
}

impl MapMemoryManager<'_> {
    pub(crate) fn submap(&mut self, subzone_level: SubzoneLevel) -> AssetState<SubmapMemory> {
        let sav_path = self.active_sav.sav_path();
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
                let index =
                    (subzone_level.z.rem_euclid(8) * 8 + subzone_level.x.rem_euclid(8)) as usize;
                AssetState::Available {
                    asset: &map_memory.0 .0[index],
                }
            }
            AssetState::Loading => AssetState::Loading,
            AssetState::Nonexistent => AssetState::Nonexistent,
        }
    }

    pub(crate) fn base_zone_level(&self, handle: &AssetId<MapMemoryAsset>) -> Option<ZoneLevel> {
        self.storage.region(handle).inspect(|zone_level| {
            assert!(
                zone_level.zone.x.rem_euclid(4) == 0 && zone_level.zone.z.rem_euclid(4) == 0,
                "Misaligned zone level: {zone_level:?}"
            );
        })
    }
}
