use crate::prelude::PosOffset;

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

impl TryFrom<PosOffset> for HorizontalDirection {
    type Error = ();

    fn try_from(value: PosOffset) -> Result<Self, Self::Error> {
        Ok(match (value.x, value.z) {
            (-1, -1) => Self::NorthWest,
            (-1, 0) => Self::West,
            (-1, 1) => Self::SouthWest,
            (0, -1) => Self::North,
            (0, 0) => Self::Here,
            (0, 1) => Self::South,
            (1, -1) => Self::NorthEast,
            (1, 0) => Self::East,
            (1, 1) => Self::SouthEast,
            _ => return Err(()),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Nbor {
    Up,
    Horizontal(HorizontalDirection),
    Down,
}

impl Nbor {
    pub(crate) const HERE: Self = Self::Horizontal(HorizontalDirection::Here);
    pub(crate) const ALL: [Self; 11] = [
        Self::Up,
        Self::Horizontal(HorizontalDirection::NorthWest),
        Self::Horizontal(HorizontalDirection::North),
        Self::Horizontal(HorizontalDirection::NorthEast),
        Self::Horizontal(HorizontalDirection::West),
        Self::HERE,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum NborDistance {
    Up,
    Adjacent,
    Diagonal,
    Zero,
    Down,
}
