use pathfinding::num_traits::Zero;
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Sub},
};
use time::OffsetDateTime;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Milliseconds(pub(crate) u64);

impl Milliseconds {
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) const MINUTE: Self = Self(60 * 1000);
    pub(crate) const EIGHT_HOURS: Self = Self(8 * 60 * 60 * 1000);
}

impl fmt::Debug for Milliseconds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.03?} s", self.0 as f32 * 0.001)
    }
}

impl Add for Milliseconds {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl AddAssign for Milliseconds {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Sub for Milliseconds {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Zero for Milliseconds {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, Default, Eq)]
pub(crate) struct Timestamp {
    /// Since start of the first year of the cataclysm
    offset: Milliseconds,
    season_length: u64,
}

impl Timestamp {
    pub(crate) const ZERO: Self = Self::new(0, 1);

    pub(crate) const fn new(turn: u64, season_length: u64) -> Self {
        Self {
            offset: Milliseconds(1000 * turn),
            season_length,
        }
    }

    pub(crate) const fn minute_of_day(&self) -> u64 {
        self.offset.0 / (1000 * 60) % (24 * 60)
    }
}

impl Add<Milliseconds> for Timestamp {
    type Output = Self;

    fn add(self, other: Milliseconds) -> Self {
        Self {
            offset: self.offset + other,
            season_length: self.season_length,
        }
    }
}

impl AddAssign<Milliseconds> for Timestamp {
    fn add_assign(&mut self, other: Milliseconds) {
        self.offset += other;
    }
}

impl Sub for Timestamp {
    type Output = Milliseconds;

    fn sub(self, other: Self) -> Milliseconds {
        self.offset - other.offset
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let deciseconds = self.offset.0 / 100;
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
