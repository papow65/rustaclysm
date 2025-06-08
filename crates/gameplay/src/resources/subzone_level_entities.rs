use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::platform::collections::{HashMap, hash_map::Entry};
use bevy::prelude::{Entity, Resource};
use gameplay_location::SubzoneLevel;

#[derive(Default, Resource)]
pub(crate) struct SubzoneLevelEntities {
    subzone_levels: HashMap<SubzoneLevel, Entity>,
    reverse: EntityHashMap<SubzoneLevel>,
}

impl SubzoneLevelEntities {
    pub(crate) fn get(&self, subzone_level: SubzoneLevel) -> Option<Entity> {
        self.subzone_levels.get(&subzone_level).copied()
    }

    pub(crate) fn add(&mut self, subzone_level: SubzoneLevel, entity: Entity) {
        let entry = self.subzone_levels.entry(subzone_level);
        assert!(
            matches!(entry, Entry::Vacant(..)),
            "There shouldn't be an existing entry"
        );
        entry.insert(entity);

        let reverse_entry = self.reverse.entry(entity);
        assert!(
            matches!(reverse_entry, Entry::Vacant(..)),
            "There shouldn't be a reverse entry"
        );
        reverse_entry.insert(subzone_level);
    }

    pub(crate) fn remove(&mut self, subzone_level: SubzoneLevel) -> Option<Entity> {
        self.subzone_levels
            .remove(&subzone_level)
            .inspect(|removed_entity| {
                self.reverse.remove(removed_entity);
            })
    }

    pub(crate) fn loaded(&self) -> bool {
        !self.subzone_levels.is_empty()
    }
}
