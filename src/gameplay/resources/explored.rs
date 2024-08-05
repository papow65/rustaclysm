use crate::gameplay::resources::map_memory_manager::MapMemoryManager;
use crate::gameplay::resources::overmap_buffer_manager::OvermapBufferManager;
use crate::prelude::{
    AssetState, Level, OvermapBuffer, Overzone, Pos, SubzoneLevel, Zone, ZoneLevel,
};
use bevy::{prelude::Resource, utils::HashMap};

/// Ever seen by the player character
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SeenFrom {
    FarAway,
    CloseBy,
    Never,
}

#[derive(Default, Resource)]
pub(crate) struct Explored {
    zone_level: HashMap<ZoneLevel, SeenFrom>,
    pos: HashMap<Pos, bool>,
    loaded_overzones: Vec<Overzone>,
}

impl Explored {
    pub(crate) fn mark_pos_seen(&mut self, pos: Pos) {
        // Lower the zone level to the ground
        let zone_level = ZoneLevel {
            zone: Zone::from(pos),
            level: pos.level.min(Level::ZERO),
        };

        self.zone_level.insert(zone_level, SeenFrom::CloseBy);
        self.pos.insert(pos, true);
    }

    pub(crate) fn has_zone_level_been_seen(
        &mut self,
        overmap_buffer_manager: &mut OvermapBufferManager,
        zone_level: ZoneLevel,
    ) -> Option<SeenFrom> {
        self.zone_level.get(&zone_level).copied().or_else(|| {
            let overzone = Overzone::from(zone_level.zone);
            if let AssetState::Available {
                asset: overmap_buffer,
            } = overmap_buffer_manager.get(overzone)
            {
                self.load_buffer(overzone, overmap_buffer);
                self.zone_level.get(&zone_level).copied()
            } else {
                None
            }
        })
    }

    pub(crate) fn has_pos_been_seen(&self, pos: Pos) -> bool {
        self.pos.get(&pos) == Some(&true)
    }

    pub(crate) fn load_buffer(&mut self, overzone: Overzone, overmap_buffer: &OvermapBuffer) {
        if !self.loaded_overzones.contains(&overzone) {
            for level in Level::GROUNDS {
                let overmap = overmap_buffer
                    .visible
                    .get(level.index())
                    .expect("level missing")
                    .load_as_overzone(overzone, level);
                for (zone_level, seen) in overmap {
                    self.zone_level.entry(zone_level).or_insert(if *seen {
                        SeenFrom::FarAway
                    } else {
                        SeenFrom::Never
                    });
                }
            }
            self.loaded_overzones.push(overzone);
        }
    }

    pub(crate) fn load_memory(
        &mut self,
        map_memory_manager: &mut MapMemoryManager,
        base_zone_level: ZoneLevel,
    ) {
        // TODO check if performant enough

        let base_subzone_level = base_zone_level.subzone_levels()[0];
        for z in 0..8 {
            for x in 0..8 {
                let subzone_level = SubzoneLevel {
                    x: base_subzone_level.x + x,
                    level: base_subzone_level.level,
                    z: base_subzone_level.z + z,
                };
                let AssetState::Available {
                    asset: submap_memory,
                } = map_memory_manager.submap(subzone_level)
                else {
                    panic!("Map memory asset not available for {subzone_level:?}");
                };
                for zz in 0..12 {
                    for xx in 0..12 {
                        if submap_memory.seen(xx, zz) {
                            self.mark_pos_seen(
                                subzone_level
                                    .base_corner()
                                    .horizontal_offset(i32::from(xx), i32::from(zz)),
                            );
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn loaded(&self) -> bool {
        !self.zone_level.is_empty()
    }
}
