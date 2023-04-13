use crate::prelude::Milliseconds;
use bevy::{
    prelude::{Entity, Resource},
    utils::HashMap,
};

#[derive(Resource)]
pub(crate) struct Timeouts {
    start: Milliseconds,
    m: HashMap<Entity, Milliseconds>,
}

impl Timeouts {
    pub(crate) fn new(turn: u64) -> Self {
        Self {
            start: Milliseconds(1000 * turn),
            m: HashMap::default(),
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, timeout: Milliseconds) {
        self.m.get_mut(&entity).unwrap().0 += timeout.0;
    }

    /// Does not 'pop' the entity
    /// Adds a timeout for untracked entities
    pub(crate) fn next(&mut self, entities: &[Entity]) -> Option<Entity> {
        self.m.retain(|e, _| entities.contains(e));
        let time = self.time();
        entities
            .iter()
            .copied()
            .min_by_key(|e| *self.m.entry(*e).or_insert(time))
    }

    pub(crate) fn time(&self) -> Milliseconds {
        self.m.values().min().copied().unwrap_or(self.start)
    }
}
