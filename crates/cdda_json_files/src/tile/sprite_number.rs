use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash, Deserialize)]
pub struct SpriteNumber(u16);

impl SpriteNumber {
    #[must_use]
    pub const fn new(number: u16) -> Self {
        Self(number)
    }

    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self.0
    }
}
