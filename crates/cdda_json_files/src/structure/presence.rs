use serde::Deserialize;
use std::num::{NonZeroI32, NonZeroU8, NonZeroU32};
use std::{fmt, hash::Hash, ops::Mul};

pub trait RequiredPart:
    Clone
    + Copy
    + fmt::Debug
    + From<NonZeroU32>
    + Mul<Self, Output = Self>
    + PartialEq
    + Eq
    + Hash
    + PartialOrd
{
    fn present() -> Self;

    fn needs_quantity(self) -> bool;

    fn format(self, string: &str) -> String;

    fn item_amount(self) -> u32;
    fn used_amount(self) -> u32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Deserialize)]
#[serde(from = "NonZeroI32")]
pub enum RequiredTool {
    Uncharged,
    Charged { charges: NonZeroU32 },
}

impl Mul<Self> for RequiredTool {
    type Output = Self;

    fn mul(self, factor: Self) -> Self {
        let (Self::Charged { charges }, Self::Charged { charges: factor }) = (self, factor) else {
            return Self::Uncharged;
        };

        Self::Charged {
            charges: factor.saturating_mul(charges),
        }
    }
}

impl RequiredPart for RequiredTool {
    fn present() -> Self {
        Self::Uncharged
    }

    fn needs_quantity(self) -> bool {
        matches!(self, Self::Charged { .. })
    }

    fn format(self, string: &str) -> String {
        if let Self::Charged { charges } = self {
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

    fn used_amount(self) -> u32 {
        if let Self::Charged { charges } = self {
            charges.get()
        } else {
            0
        }
    }
}

impl From<NonZeroI32> for RequiredTool {
    fn from(value: NonZeroI32) -> Self {
        // TODO Validate that there is no more nuanced meaing behind the apparently inconsistent usage of signs.

        let minus_one = -NonZeroI32::from(NonZeroU8::MIN);
        if value == minus_one {
            Self::Uncharged
        } else {
            Self::from(value.unsigned_abs())
        }
    }
}

impl From<NonZeroU32> for RequiredTool {
    fn from(charges: NonZeroU32) -> Self {
        Self::Charged { charges }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Deserialize)]
#[serde(from = "NonZeroU32")]
pub struct RequiredComponent {
    pub amount: NonZeroU32,
}

impl From<NonZeroU32> for RequiredComponent {
    fn from(amount: NonZeroU32) -> Self {
        Self { amount }
    }
}

impl Mul<Self> for RequiredComponent {
    type Output = Self;

    fn mul(self, factor: Self) -> Self {
        Self {
            amount: self.amount.saturating_mul(factor.amount),
        }
    }
}

impl RequiredPart for RequiredComponent {
    fn present() -> Self {
        Self {
            amount: NonZeroU32::MIN,
        }
    }

    fn needs_quantity(self) -> bool {
        true
    }

    fn format(self, string: &str) -> String {
        format!("{} {string}", self.amount)
    }

    fn item_amount(self) -> u32 {
        self.amount.get()
    }

    fn used_amount(self) -> u32 {
        self.amount.get()
    }
}

#[cfg(test)]
mod recipe_tests {
    use super::*;
    use std::num::TryFromIntError;

    #[test]
    fn tool_sorting() -> Result<(), TryFromIntError> {
        let one_charge = RequiredTool::Charged {
            charges: 1.try_into()?,
        };
        let three_charges = RequiredTool::Charged {
            charges: 3.try_into()?,
        };

        assert!(RequiredTool::Uncharged < one_charge);
        assert!(RequiredTool::Uncharged < three_charges);
        assert!(one_charge < three_charges);

        Ok(())
    }
}
