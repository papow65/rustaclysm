use crate::cdda::paths::MapMemoryPath;
use crate::cdda::region_assets::AssetStorage;
use crate::{ActiveSav, AssetState, Exploration, MapMemoryAsset, SubzoneLevel, Zone, ZoneLevel};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetEvent, AssetServer, Assets, EventReader, Res, ResMut};
use cdda_json_files::SubmapMemory;

#[derive(SystemParam)]
pub(crate) struct MapMemoryManager<'w, 's> {
    asset_events: EventReader<'w, 's, AssetEvent<MapMemoryAsset>>,
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<MapMemoryAsset>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<MapMemoryAsset>>,
}

impl MapMemoryManager<'_, '_> {
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
        //trace!("{:?}", &path);
        let map_memory =
            self.storage
                .handle(&self.asset_server, &self.assets, base_zone_level, path);
        match map_memory {
            AssetState::Available { asset: map_memory } => {
                let index =
                    (subzone_level.z.rem_euclid(8) * 8 + subzone_level.x.rem_euclid(8)) as usize;
                AssetState::Available {
                    asset: &map_memory.0.list()[index],
                }
            }
            AssetState::Loading => AssetState::Loading,
            AssetState::Nonexistent => AssetState::Nonexistent,
        }
    }

    pub(crate) fn read_seen_pos(&mut self) -> impl Iterator<Item = Exploration> + use<'_> {
        self.asset_events
            .read()
            .filter_map(|event| {
                if let AssetEvent::LoadedWithDependencies { id } = event {
                    let base_zone_level = self.storage.region(id).inspect(|zone_level| {
                        assert!(
                            zone_level.zone.x.rem_euclid(4) == 0
                                && zone_level.zone.z.rem_euclid(4) == 0,
                            "Misaligned zone level: {zone_level:?}"
                        );
                    });

                    match (base_zone_level, self.assets.get(*id)) {
                        (Some(zone_level), Some(MapMemoryAsset(map_memory))) => {
                            Some((zone_level, map_memory))
                        }
                        (None, None) => None,
                        unexpected => panic!("{unexpected:?}"),
                    }
                } else {
                    None
                }
            })
            .flat_map(|(base_zone_level, map_memory)| {
                let base_subzone_level = base_zone_level.subzone_levels()[0];
                (0..8).flat_map(move |subzone_z_offset| {
                    (0..8).map(move |subzone_x_offset| {
                        let subzone_level = SubzoneLevel {
                            x: base_subzone_level.x + subzone_x_offset,
                            level: base_subzone_level.level,
                            z: base_subzone_level.z + subzone_z_offset,
                        };
                        let index = (subzone_level.z.rem_euclid(8) * 8
                            + subzone_level.x.rem_euclid(8))
                            as usize;
                        (subzone_level, &map_memory.list()[index])
                    })
                })
            })
            .map(|(subzone_level, submap_memory)| {
                let pos = (0..12)
                    .flat_map(move |local_pos_z| {
                        (0..12).filter_map(move |local_pos_x| {
                            submap_memory.seen(local_pos_x, local_pos_z).then_some(
                                subzone_level.base_corner().horizontal_offset(
                                    i32::from(local_pos_x),
                                    i32::from(local_pos_z),
                                ),
                            )
                        })
                    })
                    .collect();
                Exploration::SubzoneLevel(subzone_level, pos)
            })
    }
}
