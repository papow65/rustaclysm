use pathfinding::num_traits::Zero;
use regex::Regex;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Div, Mul, Sub},
    sync::LazyLock,
};
use time::OffsetDateTime;

use crate::Error;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct Duration {
    milliseconds: u64,
}

impl Duration {
    pub const ZERO: Self = Self { milliseconds: 0 };
    pub const MILLISECOND: Self = Self { milliseconds: 1 };
    pub const SECOND: Self = Self {
        milliseconds: Self::MILLISECOND.milliseconds * 1000,
    };
    pub const MINUTE: Self = Self {
        milliseconds: Self::SECOND.milliseconds * 60,
    };
    pub const HOUR: Self = Self {
        milliseconds: Self::MINUTE.milliseconds * 60,
    };
    pub const DAY: Self = Self {
        milliseconds: Self::HOUR.milliseconds * 24,
    };

    #[must_use]
    pub const fn milliseconds(self) -> u64 {
        self.milliseconds
    }

    /// Euclidian division, return the quotient and keep the remainder
    pub const fn extract_div(&mut self, modulo: Self) -> u64 {
        let extracted = self.milliseconds / modulo.milliseconds;
        self.milliseconds %= modulo.milliseconds;
        extracted
    }

    #[must_use]
    pub fn short_format(self) -> String {
        format!("{self:1}")
    }
}

impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.03?} s", self.milliseconds as f32 * 0.001)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut spacing = "";
        let days = self.milliseconds / Self::DAY.milliseconds;
        if 0 < days {
            write!(f, "{days} day{}", plural(days))?;
            spacing = " ";

            if f.width() == Some(1) {
                return Ok(());
            }
        }

        let hours = (self.milliseconds % Self::DAY.milliseconds) / Self::HOUR.milliseconds;
        if 0 < hours {
            write!(f, "{spacing}{hours} hour{}", plural(hours))?;
            spacing = " ";

            if f.width() == Some(1) {
                return Ok(());
            }
        }

        let minutes = (self.milliseconds % Self::HOUR.milliseconds) / Self::MINUTE.milliseconds;
        if 0 < minutes {
            write!(f, "{spacing}{minutes} minute{}", plural(minutes))?;
            spacing = " ";

            if f.width() == Some(1) {
                return Ok(());
            }
        }

        let seconds = (self.milliseconds % Self::MINUTE.milliseconds) / Self::SECOND.milliseconds;
        if 0 < seconds {
            write!(f, "{spacing}{seconds} second{}", plural(seconds))?;
            spacing = " ";

            if f.width() == Some(1) {
                return Ok(());
            }
        }

        let milliseconds = self.milliseconds % Self::SECOND.milliseconds;
        if 0 < milliseconds {
            write!(
                f,
                "{spacing}{milliseconds} millisecond{}",
                plural(milliseconds)
            )?;
        } else if self.milliseconds == 0 {
            write!(f, "no time")?;
        }

        Ok(())
    }
}

const fn plural(i: u64) -> &'static str {
    if i == 1 { "" } else { "s" }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            milliseconds: self.milliseconds + other.milliseconds,
        }
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, other: Self) {
        self.milliseconds += other.milliseconds;
    }
}

impl Div<Self> for Duration {
    type Output = f32;

    fn div(self, div: Self) -> f32 {
        self.milliseconds as f32 / div.milliseconds as f32
    }
}

impl Div<u64> for Duration {
    type Output = Self;

    fn div(self, div: u64) -> Self {
        Self {
            milliseconds: self.milliseconds / div,
        }
    }
}

impl TryFrom<&str> for Duration {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        static DURATION_PARSER: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new("(?:([0-9]+) *([a-zA-Z]+))+").expect("Valid regex for duration")
        });

        let mut milliseconds = 0;
        for capture in DURATION_PARSER.captures_iter(value) {
            let (_full, [quantity, unit]) = capture.extract();

            let quantity = quantity.parse::<u64>()?;
            let unit = unit.to_lowercase();
            //trace!("{capture:?} {_full:?} {&quantity} {&unit}");

            let unit_factor = match unit.as_str() {
                "ms" | "millisecond" | "milliseconds" => Self::MILLISECOND.milliseconds,
                "s" | "second" | "seconds" => Self::SECOND.milliseconds,
                "m" | "minute" | "minutes" => Self::MINUTE.milliseconds,
                "h" | "hour" | "hours" => Self::HOUR.milliseconds,
                "d" | "day" | "days" => Self::DAY.milliseconds,
                _ => {
                    return Err(Error::UnknowUnit {
                        _value: String::from(value),
                    });
                }
            };
            milliseconds += quantity * unit_factor;
        }

        Ok(Self { milliseconds })
    }
}

impl TryFrom<String> for Duration {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        Self::try_from(value.as_ref())
    }
}

impl Mul<u64> for Duration {
    type Output = Self;

    fn mul(self, factor: u64) -> Self {
        Self {
            milliseconds: self.milliseconds * factor,
        }
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            milliseconds: self.milliseconds.saturating_sub(other.milliseconds),
        }
    }
}

impl Zero for Duration {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, Default, Eq)]
pub struct Timestamp {
    /// Since start of the first year of the cataclysm
    offset: Duration,
    season_length: u64,
}

impl Timestamp {
    pub const ZERO: Self = Self::new(0, 1);

    #[must_use]
    pub const fn new(turn: u64, season_length: u64) -> Self {
        Self {
            offset: Duration {
                milliseconds: turn * 1000,
            },
            season_length,
        }
    }

    #[must_use]
    pub const fn minute_of_day(&self) -> u64 {
        (self.offset.milliseconds() % Duration::DAY.milliseconds())
            / Duration::MINUTE.milliseconds()
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, other: Duration) -> Self {
        Self {
            offset: self.offset + other,
            season_length: self.season_length,
        }
    }
}

impl AddAssign<Duration> for Timestamp {
    fn add_assign(&mut self, other: Duration) {
        self.offset += other;
    }
}

impl Sub for Timestamp {
    type Output = Duration;

    fn sub(self, other: Self) -> Duration {
        self.offset - other.offset
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let deciseconds = self.offset.milliseconds / 100;
        let seconds = deciseconds / 10;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;
        let seasons = days / self.season_length;

        // based on https://cataclysmdda.org/lore-background.html
        let year = seasons / 4 + OffsetDateTime::now_utc().year() as u64 + 1;

        let season_name = match seasons % 4 {
            0 => "Spring",
            1 => "Summer",
            2 => "Autumn",
            3 => "Winter",
            _ => panic!("Modulo error"),
        };
        let day_of_season = days % self.season_length + 1; // 1-based

        let hours = hours % 24;
        let minutes = minutes % 60;
        let seconds = seconds % 60;
        let deciseconds = deciseconds % 10;

        write!(
            f,
            "{year:#04}-{season_name}-{day_of_season:#02} \
{hours:#02}:{minutes:#02}:{seconds:#02}.{deciseconds}"
        )
    }
}

impl PartialEq for Timestamp {
    fn eq(&self, other: &Self) -> bool {
        self.offset.eq(&other.offset)
    }
}

impl Hash for Timestamp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

#[cfg(test)]
mod time_tests {
    use super::*;

    #[test]
    fn add_asign_works() {
        let mut total = Duration { milliseconds: 2 };
        total += Duration { milliseconds: 3 };
        assert_eq!(total, Duration { milliseconds: 5 });
    }

    #[test]
    fn parsing_works() {
        assert_eq!(
            Duration::try_from("21 s"),
            Ok(Duration {
                milliseconds: 21 * 1000
            })
        );
        assert_eq!(
            Duration::try_from("35m"),
            Ok(Duration {
                milliseconds: 35 * 60 * 1000
            })
        );
        assert_eq!(
            Duration::try_from("31 h 40 m"),
            Ok(Duration {
                milliseconds: 31 * 60 * 60 * 1000 + 40 * 60 * 1000
            })
        );
    }

    #[test]
    fn formatting_works() {
        assert_eq!(
            Duration { milliseconds: 0 }.to_string(),
            String::from("no time")
        );
        assert_eq!(
            Duration {
                milliseconds: 21 * 1000
            }
            .to_string(),
            String::from("21 seconds")
        );
        assert_eq!(
            Duration {
                milliseconds: 35 * 60 * 1000
            }
            .to_string(),
            String::from("35 minutes")
        );
        assert_eq!(
            Duration {
                milliseconds: (((2 * 24 + 17) * 60) + 40) * 60 * 1000
            }
            .to_string(),
            String::from("2 days 17 hours 40 minutes")
        );
        assert_eq!(
            Duration {
                milliseconds: (((2 * 24 + 17) * 60) + 40) * 60 * 1000
            }
            .short_format(),
            String::from("2 days")
        );
    }
}
