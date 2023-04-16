use crate::prelude::Nbor;
use std::{cmp::Ordering, fmt};

#[derive(Clone, Debug, Eq, Hash)]
pub(crate) enum InventorySection {
    Nbor(Nbor), // not up or down
    Hands,
    Clothing,
}

impl fmt::Display for InventorySection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Nbor(n) => match n.horizontal_offset() {
                    (-1, -1) => "NW",
                    (-1, 0) => "W",
                    (-1, 1) => "SW",
                    (0, -1) => "N",
                    (0, 0) => "Here",
                    (0, 1) => "S",
                    (1, -1) => "NE",
                    (1, 0) => "E",
                    (1, 1) => "SE",
                    _ => panic!("{self:?}"),
                },
                Self::Hands => "Hands",
                Self::Clothing => "Clothing",
            }
        )
    }
}

impl Ord for InventorySection {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Nbor(a), Self::Nbor(b)) => a.horizontal_offset().cmp(&b.horizontal_offset()),
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

impl PartialEq for InventorySection {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
