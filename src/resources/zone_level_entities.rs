use crate::prelude::ZoneLevel;
use bevy::{
    prelude::{Entity, Resource},
    utils::{Entry, HashMap},
};

#[derive(Default, Resource)]
pub(crate) struct ZoneLevelEntities {
    zone_levels: HashMap<ZoneLevel, Entity>,
    reverse: HashMap<Entity, ZoneLevel>,
}

impl ZoneLevelEntities {
    pub(crate) fn get(&self, zone_level: ZoneLevel) -> Option<Entity> {
        self.zone_levels.get(&zone_level).copied()
    }

    pub(crate) fn add(&mut self, zone_level: ZoneLevel, entity: Entity) {
        let entry = self.zone_levels.entry(zone_level);
        assert!(
            matches!(entry, Entry::Vacant(..)),
            "Duplicate for {entry:?} - new: {entity:?}"
        );
        entry.insert(entity);

        let reverse_entry = self.reverse.entry(entity);
        assert!(matches!(reverse_entry, Entry::Vacant(..)));
        reverse_entry.insert(zone_level);
    }

    #[allow(unused)]
    pub(crate) fn remove(&mut self, entity: Entity) {
        let zone_level = self.reverse.remove(&entity).unwrap();
        let removed = self.zone_levels.remove(&zone_level);
        assert!(removed.is_some());
    }
}
