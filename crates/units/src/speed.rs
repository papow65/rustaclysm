use std::{fmt, ops::Mul};

#[derive(Clone, Copy)]
pub struct Speed {
    millimeter_per_second: u64,
}

impl Speed {
    #[must_use]
    pub const fn from_kmph(n: f32) -> Self {
        Self {
            millimeter_per_second: (n * 1_000_000.0 / 3_600.0) as u64,
        }
    }

    #[must_use]
    pub const fn millimeter_per_second(&self) -> u64 {
        self.millimeter_per_second
    }

    #[must_use]
    pub const fn as_kmph(&self) -> f32 {
        self.millimeter_per_second as f32 * 3_600.0 / 1_000_000.0
    }

    #[must_use]
    pub const fn combine(&self, other: Self, average: Self) -> Self {
        Self {
            millimeter_per_second: self.millimeter_per_second * other.millimeter_per_second
                / average.millimeter_per_second,
        }
    }
}

impl fmt::Debug for Speed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.01?} km/h",
            self.millimeter_per_second as f32 * 3_600.0 / 1_000_000.0
        )
    }
}

impl Mul<f32> for Speed {
    type Output = Self;

    fn mul(self, value: f32) -> Self {
        Self {
            millimeter_per_second: (self.millimeter_per_second as f32 * value) as u64,
        }
    }
}
