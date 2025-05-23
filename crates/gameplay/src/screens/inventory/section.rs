use crate::HorizontalDirection;
use std::{cmp::Ordering, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum InventorySection {
    Nbor(HorizontalDirection),
    Hands,
    Clothing,
}

impl fmt::Display for InventorySection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{} ({})]",
            match self {
                Self::Nbor(direction) => direction.succinct(),
                Self::Hands => "Hands",
                Self::Clothing => "Clothing",
            },
            match self {
                Self::Nbor(direction) => (48 + direction.numpad()) as char,
                Self::Hands => 'H',
                Self::Clothing => 'C',
            }
        )
    }
}

impl Ord for InventorySection {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Nbor(a), Self::Nbor(b)) => reverse(a.offset()).cmp(&reverse(b.offset())),
            (Self::Nbor(_) | Self::Hands, Self::Clothing) | (Self::Nbor(_), Self::Hands) => {
                Ordering::Less
            }
            (Self::Hands, Self::Hands) | (Self::Clothing, Self::Clothing) => Ordering::Equal,
            (Self::Hands, Self::Nbor(_)) | (Self::Clothing, Self::Nbor(_) | Self::Hands) => {
                Ordering::Greater
            }
        }
    }
}

impl PartialOrd for InventorySection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const fn reverse((a, b): (i32, i32)) -> (i32, i32) {
    (b, a)
}
