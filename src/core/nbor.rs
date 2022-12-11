use crate::prelude::{Millimeter, WalkingDistance};

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct HorizontalNborOffset {
    /*private*/ x: i32, // -1, 0, or 1
    /*private*/ z: i32, // -1, 0, or 1
}

impl HorizontalNborOffset {
    fn try_from(x: i32, z: i32) -> Option<HorizontalNborOffset> {
        if x.abs().max(z.abs()) == 1 {
            Some(HorizontalNborOffset { x, z })
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Nbor {
    Up,
    Horizontal(HorizontalNborOffset),
    Here,
    Down,
}

impl Nbor {
    pub(crate) const ALL: [Self; 11] = [
        Self::Up,
        Self::Horizontal(HorizontalNborOffset { x: 1, z: 0 }),
        Self::Horizontal(HorizontalNborOffset { x: 1, z: 1 }),
        Self::Horizontal(HorizontalNborOffset { x: 0, z: 1 }),
        Self::Horizontal(HorizontalNborOffset { x: -1, z: 1 }),
        Self::Horizontal(HorizontalNborOffset { x: -1, z: 0 }),
        Self::Horizontal(HorizontalNborOffset { x: -1, z: -1 }),
        Self::Horizontal(HorizontalNborOffset { x: 0, z: -1 }),
        Self::Horizontal(HorizontalNborOffset { x: 1, z: -1 }),
        Self::Here,
        Self::Down,
    ];

    pub(crate) fn try_horizontal(x: i32, z: i32) -> Option<Self> {
        HorizontalNborOffset::try_from(x, z).map(Self::Horizontal)
    }

    pub(crate) const fn horizontal_offset(&self) -> (i32, i32) {
        match self {
            Self::Horizontal(HorizontalNborOffset { x, z }) => (*x, *z),
            _ => (0, 0),
        }
    }

    pub(crate) fn distance(&self) -> WalkingDistance {
        match self {
            Self::Up => WalkingDistance {
                horizontal: Millimeter(0),
                up: Millimeter::VERTICAL,
                down: Millimeter(0),
            },
            Self::Down => WalkingDistance {
                horizontal: Millimeter(0),
                up: Millimeter(0),
                down: Millimeter::VERTICAL,
            },
            horizontal => {
                let (x, z) = horizontal.horizontal_offset();
                WalkingDistance {
                    horizontal: if x == 0 || z == 0 {
                        Millimeter::ADJACENT
                    } else {
                        Millimeter::DIAGONAL
                    },
                    up: Millimeter(0),
                    down: Millimeter(0),
                }
            }
        }
    }
}
