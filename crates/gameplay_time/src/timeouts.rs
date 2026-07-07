use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::prelude::{Entity, Resource};
use units::{Duration, Timestamp};

/// Resource tracking local timeouts and turn timestamps for game entities.
#[derive(Resource)]
pub struct Timeouts {
    start: Timestamp,
    m: EntityHashMap<Timestamp>,
}

impl Timeouts {
    /// Creates a new timeouts tracker initialized with a start timestamp.
    #[must_use]
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            start: timestamp,
            m: EntityHashMap::default(),
        }
    }

    /// Adds a timeout duration to a specific entity.
    pub fn add(&mut self, entity: Entity, timeout: Duration) {
        *self.m.get_mut(&entity).expect("entity should be known") += timeout;
    }

    /// Gets the next entity that is due for an action.
    /// Does not 'pop' the entity.
    /// Adds a timeout for untracked entities.
    #[must_use]
    pub fn next(&mut self, entities: &[Entity]) -> Option<Entity> {
        self.m.retain(|e, _| entities.contains(e));
        let time = self.max_timestamp();
        entities
            .iter()
            .min_by_key(|&e| (*self.m.entry(*e).or_insert(time), e))
            .copied()
    }

    /// Calculates the minimum timestamp among all active timeouts, defaulting to start time.
    pub(crate) fn max_timestamp(&self) -> Timestamp {
        self.m.values().min().copied().unwrap_or(self.start)
    }

    /// Checks if the player is the next entity to take a turn.
    #[must_use]
    pub fn is_player_next(&self, player: Entity, entities: &[Entity]) -> bool {
        let Some(player) = self.m.get(&player) else {
            return true;
        };

        entities
            .iter()
            .all(|entity| self.m.get(entity).is_none_or(|other| other <= player))
    }
}
