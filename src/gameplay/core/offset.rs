#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct LevelOffset {
    pub(crate) h: i8,
}

impl LevelOffset {
    pub(crate) const ZERO: Self = Self { h: 0 };
    pub(crate) const UP: Self = Self { h: 1 };
    pub(crate) const DOWN: Self = Self { h: -1 };
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct PosOffset {
    pub(crate) x: i32,
    pub(crate) level: LevelOffset,
    pub(crate) z: i32,
}

impl PosOffset {
    pub(crate) fn player_hint(&self) -> &str {
        if self.x == 0 && self.z == 0 {
            match self.level {
                LevelOffset { h } if h > 0 => "U",
                LevelOffset::ZERO => "H",
                _ => "D",
            }
        } else if 2 * self.z.abs() <= self.x.abs() {
            if 0 < self.x {
                "E"
            } else {
                "W"
            }
        } else if 2 * self.x.abs() <= self.z.abs() {
            if 0 < self.z {
                "S"
            } else {
                "N"
            }
        } else if 0 < self.x {
            if 0 < self.z {
                "SE"
            } else {
                assert!(self.z < 0, "Unexpected offset: {self:?}");
                "NE"
            }
        } else {
            assert!(self.x < 0, "Unexpected offset: {self:?}");
            if 0 < self.z {
                "SW"
            } else {
                assert!(self.z < 0, "Unexpected offset: {self:?}");
                "NW"
            }
        }
    }
}
