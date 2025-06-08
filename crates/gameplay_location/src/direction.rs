use crate::PosOffset;
use strum::VariantArray;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantArray)]
pub enum CardinalDirection {
    North,
    West,
    East,
    South,
}

impl TryFrom<HorizontalDirection> for CardinalDirection {
    type Error = ();

    fn try_from(value: HorizontalDirection) -> Result<Self, Self::Error> {
        Ok(match value {
            HorizontalDirection::North => Self::North,
            HorizontalDirection::West => Self::West,
            HorizontalDirection::South => Self::South,
            HorizontalDirection::East => Self::East,
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantArray)]
pub enum HorizontalDirection {
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
    /// (x: \[-1, 0, or 1\], z: \[-1, 0, or 1\])
    #[must_use]
    pub const fn offset(self) -> (i32, i32) {
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

    #[must_use]
    pub const fn numpad(self) -> u8 {
        let (x, y) = self.offset();
        (x + 2) as u8 + 3 * (y + 1) as u8
    }

    #[must_use]
    pub const fn succinct(self) -> &'static str {
        match self {
            Self::NorthWest => "NW",
            Self::North => "N",
            Self::NorthEast => "NE",
            Self::West => "W",
            Self::Here => "Here",
            Self::East => "E",
            Self::SouthWest => "SW",
            Self::South => "S",
            Self::SouthEast => "SE",
        }
    }
}

impl From<CardinalDirection> for HorizontalDirection {
    fn from(value: CardinalDirection) -> Self {
        match value {
            CardinalDirection::North => Self::North,
            CardinalDirection::West => Self::West,
            CardinalDirection::South => Self::South,
            CardinalDirection::East => Self::East,
        }
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
