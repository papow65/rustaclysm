use serde::Deserialize;

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Hash, Deserialize)]
pub struct SpriteNumber(u16);

impl SpriteNumber {
    pub const fn new(number: u16) -> Self {
        Self(number)
    }

    pub const fn to_u16(self) -> u16 {
        self.0
    }
}
