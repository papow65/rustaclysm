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
