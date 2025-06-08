use crate::HorizontalDirection;
use bevy::input::keyboard::KeyCode;
use keyboard::Key;

/// Neighbor
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Nbor {
    Up,
    Horizontal(HorizontalDirection),
    Down,
}

impl Nbor {
    pub const HERE: Self = Self::Horizontal(HorizontalDirection::Here);
    pub const ALL: [Self; 11] = [
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

    #[must_use]
    pub fn distance(self) -> NborDistance {
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

impl From<Nbor> for Key {
    fn from(nbor: Nbor) -> Self {
        match nbor {
            Nbor::Up => Self::Character('<'),
            Nbor::Horizontal(HorizontalDirection::SouthWest) => Self::Code(KeyCode::Numpad1),
            Nbor::Horizontal(HorizontalDirection::South) => Self::Code(KeyCode::Numpad2),
            Nbor::Horizontal(HorizontalDirection::SouthEast) => Self::Code(KeyCode::Numpad3),
            Nbor::Horizontal(HorizontalDirection::West) => Self::Code(KeyCode::Numpad4),
            Nbor::HERE => Self::Code(KeyCode::Numpad5),
            Nbor::Horizontal(HorizontalDirection::East) => Self::Code(KeyCode::Numpad6),
            Nbor::Horizontal(HorizontalDirection::NorthWest) => Self::Code(KeyCode::Numpad7),
            Nbor::Horizontal(HorizontalDirection::North) => Self::Code(KeyCode::Numpad8),
            Nbor::Horizontal(HorizontalDirection::NorthEast) => Self::Code(KeyCode::Numpad9),
            Nbor::Down => Self::Character('>'),
        }
    }
}

impl TryFrom<Nbor> for HorizontalDirection {
    type Error = ();

    fn try_from(value: Nbor) -> Result<Self, Self::Error> {
        match value {
            Nbor::Horizontal(horizontal) => Ok(horizontal),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NborDistance {
    Up,
    Adjacent,
    Diagonal,
    Zero,
    Down,
}
