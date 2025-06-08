use crate::{LevelOffset, PosOffset};
use std::ops::RangeInclusive;
use units::Distance;

/// Not `Eq`, `PartialOrd`, and `Ord`, because there are multiple interpretations possible
#[derive(Clone, PartialEq)]
pub struct VisionDistance {
    square_mm: u64,
}

impl VisionDistance {
    pub const MAX_VISION_TILES: i32 = 60;
    pub const MAX_VISION_RANGE: RangeInclusive<i32> =
        -Self::MAX_VISION_TILES..=Self::MAX_VISION_TILES;

    /// Useful for estimates
    #[must_use]
    pub fn f32(&self) -> f32 {
        (self.square_mm as f32).sqrt()
    }

    /// Often slower than `in_range`
    #[must_use]
    pub fn as_tiles(&self) -> usize {
        (self.square_mm as f32 / Distance::ADJACENT.millimeter().pow(2) as f32)
            .sqrt()
            .floor() as usize
    }

    /// Often faster than `as_tiles`
    #[must_use]
    pub fn in_range(&self, max_tiles: usize) -> bool {
        let certainly_in_range = self.square_mm <= Self::from_tiles(max_tiles as i32).square_mm;
        if certainly_in_range {
            return true;
        }

        let certainly_outside_range =
            Self::from_tiles(max_tiles as i32 + 1).square_mm < self.square_mm;

        !certainly_outside_range && self.as_tiles() <= max_tiles
    }

    #[must_use]
    pub const fn from_offset(pos_offset: PosOffset) -> Self {
        Self {
            square_mm: Distance::ADJACENT.millimeter().pow(2)
                * (pos_offset.x.abs_diff(0) as u64).pow(2)
                + Distance::VERTICAL.millimeter().pow(2)
                    * (pos_offset.level.h.abs_diff(0) as u64).pow(2)
                + Distance::ADJACENT.millimeter().pow(2) * (pos_offset.z.abs_diff(0) as u64).pow(2),
        }
    }

    #[must_use]
    const fn from_tiles(tiles: i32) -> Self {
        Self::from_offset(PosOffset {
            x: tiles,
            level: LevelOffset::ZERO,
            z: 0,
        })
    }
}
