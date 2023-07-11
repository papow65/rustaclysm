#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum HorizontalDirection {
    NorthWest,
    North,
    NorthEast,
    West,
    Here,
    East,
    SouthWest,
    South,
    SouthEast,
}

impl HorizontalDirection {
    /** (x: \[-1, 0, or 1\], z: \[-1, 0, or 1\]) */
    pub(crate) const fn offset(&self) -> (i32, i32) {
        (
            match self {
                Self::NorthWest | Self::West | Self::SouthWest => -1,
                Self::North | Self::Here | Self::South => 0,
                Self::NorthEast | Self::East | Self::SouthEast => 1,
            },
            match self {
                Self::NorthWest | Self::North | Self::NorthEast => -1,
                Self::West | Self::Here | Self::East => 0,
                Self::SouthWest | Self::South | Self::SouthEast => 1,
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Nbor {
    Up,
    Horizontal(HorizontalDirection),
    Down,
}

impl Nbor {
    pub(crate) const ALL: [Self; 11] = [
        Self::Up,
        Self::Horizontal(HorizontalDirection::NorthWest),
        Self::Horizontal(HorizontalDirection::North),
        Self::Horizontal(HorizontalDirection::NorthEast),
        Self::Horizontal(HorizontalDirection::West),
        Self::Horizontal(HorizontalDirection::Here),
        Self::Horizontal(HorizontalDirection::East),
        Self::Horizontal(HorizontalDirection::SouthWest),
        Self::Horizontal(HorizontalDirection::South),
        Self::Horizontal(HorizontalDirection::SouthEast),
        Self::Down,
    ];

    pub(crate) const fn horizontal_projection(&self) -> HorizontalDirection {
        match self {
            Self::Horizontal(horizontal) => *horizontal,
            _ => HorizontalDirection::Here,
        }
    }

    pub(crate) fn distance(&self) -> NborDistance {
        match &self {
            Self::Up => NborDistance::Up,
            Self::Horizontal(horizontal) => {
                let (x, z) = horizontal.offset();
                match x.unsigned_abs() + z.unsigned_abs() {
                    0 => NborDistance::Zero,
                    1 => NborDistance::Adjacent,
                    2 => NborDistance::Diagonal,
                    _ => panic!("{x} {z}"),
                }
            }
            Self::Down => NborDistance::Down,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum NborDistance {
    Up,
    Adjacent,
    Diagonal,
    Zero,
    Down,
}
