use crate::Timeouts;
use bevy::ecs::system::SystemParam;
use bevy::prelude::Res;
use units::Timestamp;

/// The clock resource querying the current game time and environment properties.
#[derive(SystemParam)]
pub struct Clock<'w> {
    timeouts: Res<'w, Timeouts>,
}

impl Clock<'_> {
    /// Returns the current maximum timestamp tracked by the game timeouts.
    #[must_use]
    pub fn time(&self) -> Timestamp {
        self.timeouts.max_timestamp()
    }

    /// Roughly matches New England, centered around 1 PM.
    ///
    /// Source: <https://www.suncalc.org>
    #[must_use]
    pub fn sunlight_percentage(&self) -> f32 {
        // Calculation in minutes

        const SOLAR_NOON: f32 = 13.0 * 60.0;
        // We can ignore calculation errors related to solar midnight, because there is no sunlight around that time.

        let time = self.time();
        let solar_summer = time.solar_summer();

        let full_sun_diff = (1.0 + 4.0 * solar_summer) * 60.0; // Full daylight for 1-5 hours away from solar noon
        let sunset_diff = (5.5 + 3.0 * solar_summer) * 60.0; // No daylight more than 5.5-8.5 hours away from solar noon

        let minutes_from_noon = (SOLAR_NOON - time.minute_of_day() as f32).abs();

        ((sunset_diff - minutes_from_noon) / (sunset_diff - full_sun_diff)).clamp(0.0, 1.0)
    }
}
