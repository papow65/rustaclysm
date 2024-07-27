#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Hash)]
pub(crate) struct SpriteNumber(pub(super) usize);

impl SpriteNumber {
    pub(super) fn from_json(value: &serde_json::Value) -> Self {
        Self(
            value
                .as_u64()
                .expect("JSON value should be an integer (>= 0)") as usize,
        )
    }

    pub(super) fn from_number(n: &serde_json::Number) -> Self {
        Self(n.as_u64().expect("JSON value should be an integer (>= 0)") as usize)
    }

    pub(crate) const fn to_usize(self) -> usize {
        self.0
    }
}
