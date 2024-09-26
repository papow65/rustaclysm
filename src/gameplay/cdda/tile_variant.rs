use crate::gameplay::CardinalDirection;
use cdda_json_files::CddaTileVariant;

#[derive(Clone, Copy, Debug)]
pub(crate) enum TileVariant {
    // For vehicle parts:
    Broken,
    Open,
    // For terrain:
    Center,
    NorthEastCorner,
    NorthWestCorner,
    SouthhEastCorner,
    SouthWestCorner,
    NorthSouthEdge,
    EastWestEdge,
    TConnection(CardinalDirection),
    EndPiece(CardinalDirection),
    Unconnected,
}

impl TileVariant {
    pub(crate) const fn expected_length(&self) -> Option<usize> {
        Some(match self {
            Self::NorthSouthEdge | Self::EastWestEdge => 2,
            Self::NorthEastCorner
            | Self::NorthWestCorner
            | Self::SouthhEastCorner
            | Self::SouthWestCorner
            | Self::TConnection(_)
            | Self::EndPiece(_) => 4,
            _ => {
                return None;
            }
        })
    }

    pub(crate) const fn index(&self) -> Option<usize> {
        Some(match self {
            Self::NorthSouthEdge
            | Self::SouthhEastCorner
            | Self::TConnection(CardinalDirection::North)
            | Self::EndPiece(CardinalDirection::South) => 0,
            Self::EastWestEdge
            | Self::NorthEastCorner
            | Self::TConnection(CardinalDirection::West)
            | Self::EndPiece(CardinalDirection::East) => 1,
            Self::NorthWestCorner
            | Self::TConnection(CardinalDirection::South)
            | Self::EndPiece(CardinalDirection::North) => 2,
            Self::SouthWestCorner
            | Self::TConnection(CardinalDirection::East)
            | Self::EndPiece(CardinalDirection::West) => 3,
            _ => {
                return None;
            }
        })
    }
}

impl From<TileVariant> for CddaTileVariant {
    fn from(source: TileVariant) -> Self {
        match source {
            TileVariant::Broken => Self::Broken,
            TileVariant::Open => Self::Open,
            TileVariant::Center => Self::Center,
            TileVariant::NorthEastCorner
            | TileVariant::NorthWestCorner
            | TileVariant::SouthhEastCorner
            | TileVariant::SouthWestCorner => Self::Corner,
            TileVariant::NorthSouthEdge | TileVariant::EastWestEdge => Self::Edge,
            TileVariant::TConnection(_) => Self::TConnection,
            TileVariant::EndPiece(_) => Self::End,
            TileVariant::Unconnected => Self::Unconnected,
        }
    }
}
