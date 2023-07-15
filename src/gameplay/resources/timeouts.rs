use crate::prelude::{Milliseconds, Timestamp};
use bevy::{
    ecs::system::SystemParam,
    prelude::{Entity, Res, Resource},
    utils::HashMap,
};

#[derive(Resource)]
pub(crate) struct Timeouts {
    start: Timestamp,
    m: HashMap<Entity, Timestamp>,
}

impl Timeouts {
    pub(crate) fn new(timestamp: Timestamp) -> Self {
        Self {
            start: timestamp,
            m: HashMap::default(),
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, timeout: Milliseconds) {
        *self.m.get_mut(&entity).unwrap() += timeout;
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

    /** Private, use Clock.time */
    fn time(&self) -> Timestamp {
        self.m.values().min().copied().unwrap_or(self.start)
    }
}

#[derive(SystemParam)]
pub(crate) struct Clock<'w> {
    timeouts: Res<'w, Timeouts>,
}

impl<'w> Clock<'w> {
    pub(crate) fn time(&self) -> Timestamp {
        self.timeouts.time()
    }

    /** Roughly matches New England, centered around 1 PM, ignoring seasons

    Source: https://www.suncalc.org */
    pub(crate) fn sunlight_percentage(&self) -> f32 {
        // Calculation in minutes

        const SOLAR_NOON: u64 = 13 * 60;
        // We can ignore calculation errors related to solar midnight, because there is no sunlight around dat time.

        const FULL_SUN_DIFF: u64 = 3 * 60; // Full daylight up to than 3 hours away from solar noon
        const SUNSET_DIFF: u64 = 7 * 60; // No daylight more than 7 hours away from solar noon

        let minutes_from_noon = SOLAR_NOON.abs_diff(self.time().minute_of_day());

        (SUNSET_DIFF.saturating_sub(minutes_from_noon) as f32
            / SUNSET_DIFF.abs_diff(FULL_SUN_DIFF) as f32)
            .clamp(0.0, 1.0)
    }
}
