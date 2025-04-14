use bevy_log::warn;
use serde::Deserialize;
use std::{fmt, hash::Hash, num::NonZeroU32, ops::Mul};

pub trait RequiredPresence:
    Clone
    + Copy
    + fmt::Debug
    + From<u32>
    + Mul<Self, Output = Self>
    + PartialEq
    + Eq
    + Hash
    + PartialOrd
{
    fn present() -> Self;

    fn quantity_present(self) -> Option<NonZeroU32>;

    fn format(self, string: &str) -> String;

    fn item_amount(self) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Deserialize)]
#[serde(from = "i32")]
pub enum ToolPresence {
    Missing,
    Present { charges: u32 },
}

impl Mul<Self> for ToolPresence {
    type Output = Self;

    fn mul(self, factor: Self) -> Self {
        let (Self::Present { charges }, Self::Present { charges: factor }) = (self, factor) else {
            unreachable!("{:?} {:?}", self, factor)
        };

        Self::Present {
            charges: factor * charges,
        }
    }
}

impl RequiredPresence for ToolPresence {
    fn present() -> Self {
        Self::Present { charges: 0 }
    }

    fn quantity_present(self) -> Option<NonZeroU32> {
        if let Self::Present { charges } = self {
            charges.try_into().ok()
        } else {
            None
        }
    }

    fn format(self, string: &str) -> String {
        if let Some(charges) = self.quantity_present() {
            format!(
                "{string} with {charges} charge{}",
                if charges.get() == 1 { "" } else { "s" }
            )
        } else {
            String::from(string)
        }
    }

    fn item_amount(self) -> u32 {
        1 // One tool
    }
}

impl From<i32> for ToolPresence {
    fn from(value: i32) -> Self {
        // TODO Validate that there is no more nuanced meaing behind the apparently inconsistent usage of signs.

        if value == 0 {
            warn!("0 is not expected value for ToolPresence");
            Self::Present { charges: 0 }
        } else if value == -1 {
            Self::Present { charges: 0 }
        } else {
            Self::Present {
                charges: value.unsigned_abs(),
            }
        }
    }
}

impl From<u32> for ToolPresence {
    fn from(charges: u32) -> Self {
        Self::Present { charges }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Deserialize)]
#[serde(from = "u32")]
pub struct ComponentPresence {
    pub amount: u32,
}

impl From<u32> for ComponentPresence {
    fn from(amount: u32) -> Self {
        Self { amount }
    }
}

impl Mul<Self> for ComponentPresence {
    type Output = Self;

    fn mul(self, factor: Self) -> Self {
        Self {
            amount: factor.amount * self.amount,
        }
    }
}

impl RequiredPresence for ComponentPresence {
    fn present() -> Self {
        Self { amount: 1 }
    }

    fn quantity_present(self) -> Option<NonZeroU32> {
        self.amount.try_into().ok()
    }

    fn format(self, string: &str) -> String {
        format!("{} {string}", self.amount)
    }

    fn item_amount(self) -> u32 {
        self.amount
    }
}

#[cfg(test)]
mod recipe_tests {
    use super::*;

    #[test]
    fn tool_sorting() {
        assert!(ToolPresence::Missing < ToolPresence::Present { charges: 0 });
        assert!(ToolPresence::Present { charges: 0 } < ToolPresence::Present { charges: 3 });
        assert!(ToolPresence::Missing < ToolPresence::Present { charges: 3 });

        assert!(ToolPresence::Present { charges: 0 } > ToolPresence::Missing);
        assert!(ToolPresence::Present { charges: 3 } > ToolPresence::Present { charges: 0 });
        assert!(ToolPresence::Present { charges: 3 } > ToolPresence::Missing);
    }
}
