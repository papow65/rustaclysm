use serde::Deserialize;

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Hash, Deserialize)]
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
