use crate::prelude::HorizontalDirection;
use std::{cmp::Ordering, fmt};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum InventorySection {
    Nbor(HorizontalDirection),
    Hands,
    Clothing,
}

impl fmt::Display for InventorySection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Nbor(n) => match n.offset() {
                    (-1, -1) => "NW (7)",
                    (-1, 0) => "W (4)",
                    (-1, 1) => "SW (1)",
                    (0, -1) => "N (8)",
                    (0, 0) => "Here (5)",
                    (0, 1) => "S (2)",
                    (1, -1) => "NE (9)",
                    (1, 0) => "E (6)",
                    (1, 1) => "SE (3)",
                    _ => panic!("{self:?}"),
                },
                Self::Hands => "Hands (H)",
                Self::Clothing => "Clothing (C)",
            }
        )
    }
}

impl Ord for InventorySection {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Nbor(a), Self::Nbor(b)) => a.offset().cmp(&b.offset()),
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
