use serde::Deserialize;

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Hash, Deserialize)]
pub struct SpriteNumber(pub(super) usize);

impl SpriteNumber {
    pub const fn new(number: usize) -> Self {
        Self(number)
    }

    pub const fn to_usize(self) -> usize {
        self.0
    }
}
