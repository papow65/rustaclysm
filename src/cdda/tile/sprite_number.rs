use serde::Deserialize;

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Hash, Deserialize)]
pub(crate) struct SpriteNumber(pub(super) usize);

impl SpriteNumber {
    pub(crate) const fn to_usize(self) -> usize {
        self.0
    }
}
