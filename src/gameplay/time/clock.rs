use crate::gameplay::Timeouts;
use bevy::ecs::system::SystemParam;
use bevy::prelude::Res;
use units::Timestamp;

#[derive(SystemParam)]
pub(crate) struct Clock<'w> {
    timeouts: Res<'w, Timeouts>,
}

impl Clock<'_> {
    pub(crate) fn time(&self) -> Timestamp {
        self.timeouts.max_timestamp()
    }

    /// Roughly matches New England, centered around 1 PM, ignoring seasons
    ///
    /// Source: <https://www.suncalc.org>
    pub(crate) fn sunlight_percentage(&self) -> f32 {
        // Calculation in minutes

        const SOLAR_NOON: u64 = 13 * 60;
        // We can ignore calculation errors related to solar midnight, because there is no sunlight around that time.

        const FULL_SUN_DIFF: u64 = 3 * 60; // Full daylight up to 3 hours away from solar noon
        const SUNSET_DIFF: u64 = 7 * 60; // No daylight more than 7 hours away from solar noon

        let minutes_from_noon = SOLAR_NOON.abs_diff(self.time().minute_of_day());

        (SUNSET_DIFF.saturating_sub(minutes_from_noon) as f32
            / SUNSET_DIFF.abs_diff(FULL_SUN_DIFF) as f32)
            .clamp(0.0, 1.0)
    }
}
