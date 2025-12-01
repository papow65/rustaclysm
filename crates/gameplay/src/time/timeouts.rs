use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::prelude::{Entity, Resource};
use units::{Duration, Timestamp};

#[derive(Resource)]
pub(crate) struct Timeouts {
    start: Timestamp,
    m: EntityHashMap<Timestamp>,
}

impl Timeouts {
    pub(crate) fn new(timestamp: Timestamp) -> Self {
        Self {
            start: timestamp,
            m: EntityHashMap::default(),
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, timeout: Duration) {
        *self.m.get_mut(&entity).expect("entity should be known") += timeout;
    }

    /// Does not 'pop' the entity
    /// Adds a timeout for untracked entities
    pub(crate) fn next(&mut self, entities: &[Entity]) -> Option<Entity> {
        self.m.retain(|e, _| entities.contains(e));
        let time = self.max_timestamp();
        entities
            .iter()
            .min_by_key(|&e| (*self.m.entry(*e).or_insert(time), e))
            .copied()
    }

    pub(super) fn max_timestamp(&self) -> Timestamp {
        self.m.values().min().copied().unwrap_or(self.start)
    }

    pub(crate) fn is_player_next(&self, player: Entity, entities: &[Entity]) -> bool {
        let Some(player) = self.m.get(&player) else {
            return true;
        };

        entities
            .iter()
            .all(|entity| self.m.get(entity).is_none_or(|other| other <= player))
    }
}
