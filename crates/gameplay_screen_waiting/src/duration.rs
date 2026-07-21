use strum::VariantArray;
use units::{Duration, Timestamp};

#[derive(Clone, Debug, VariantArray)]
pub(super) enum WaitDuration {
    OneMinute,
    FiveMinutes,
    ThirtyMinutes,
    UntilMidnight,
    UntilMorning,
    UntilNoon,
    UntilEvening,
}

impl WaitDuration {
    pub(super) const fn message(&self) -> &str {
        match self {
            Self::OneMinute => "Wait 1 minute",
            Self::FiveMinutes => "Wait 5 minutes",
            Self::ThirtyMinutes => "Wait 30 minutes",
            Self::UntilMidnight => "Wait until midnight",
            Self::UntilMorning => "Wait until morning",
            Self::UntilNoon => "Wait until noon",
            Self::UntilEvening => "Wait until evening",
        }
    }

    pub(super) const fn key(&self) -> char {
        match self {
            Self::OneMinute => 'm',
            Self::FiveMinutes => 'f',
            Self::ThirtyMinutes => 't',
            Self::UntilMidnight => 'M',
            Self::UntilMorning => 'd',
            Self::UntilNoon => 'n',
            Self::UntilEvening => 'e',
        }
    }

    pub(super) fn until(&self, now: Timestamp) -> Timestamp {
        match self {
            Self::OneMinute => now + Duration::MINUTE,
            Self::FiveMinutes => now + Duration::MINUTE * 5,
            Self::ThirtyMinutes => now + Duration::MINUTE * 30,
            Self::UntilMidnight => now.start_of_day() + Duration::DAY,
            Self::UntilMorning => next_moment_of_day(now, Duration::DAY / 4),
            Self::UntilNoon => next_moment_of_day(now, Duration::DAY / 2),
            Self::UntilEvening => next_moment_of_day(now, Duration::DAY * 3 / 4),
        }
    }
}

fn next_moment_of_day(now: Timestamp, after_midnight: Duration) -> Timestamp {
    assert!(
        after_midnight < Duration::DAY,
        "The given moment should be less than a day"
    );

    let current_day_moment = now.start_of_day() + after_midnight;

    if now.minute_of_day() * Duration::MINUTE.milliseconds() < after_midnight.milliseconds() {
        // before moment of current day
        current_day_moment
    } else {
        // after moment of current day
        current_day_moment + Duration::DAY
    }
}
