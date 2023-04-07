use crate::prelude::SubzoneLevel;
use bevy::{
    prelude::{Entity, Resource},
    utils::{Entry, HashMap},
};

// TODO Are the tracked entities useful somewhere?
#[derive(Default, Resource)]
pub(crate) struct SubzoneLevelEntities {
    subzone_levels: HashMap<SubzoneLevel, Entity>,
    reverse: HashMap<Entity, SubzoneLevel>,
}

impl SubzoneLevelEntities {
    pub(crate) fn get(&self, subzone_level: SubzoneLevel) -> Option<Entity> {
        self.subzone_levels.get(&subzone_level).copied()
    }

    pub(crate) fn add(&mut self, subzone_level: SubzoneLevel, entity: Entity) {
        let entry = self.subzone_levels.entry(subzone_level);
        assert!(matches!(entry, Entry::Vacant(..)));
        entry.insert(entity);

        let reverse_entry = self.reverse.entry(entity);
        assert!(matches!(reverse_entry, Entry::Vacant(..)));
        reverse_entry.insert(subzone_level);
    }

    pub(crate) fn remove(&mut self, entity: Entity) {
        let subzone_level = self.reverse.remove(&entity).unwrap();
        let removed = self.subzone_levels.remove(&subzone_level);
        assert!(removed.is_some());
    }

    pub(crate) fn loaded(&self) -> bool {
        !self.subzone_levels.is_empty()
    }
}
