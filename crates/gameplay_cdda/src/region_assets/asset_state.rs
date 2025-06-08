#[derive(Debug)]
#[must_use]
pub enum AssetState<'a, T> {
    Available { asset: &'a T },
    Loading,
    Nonexistent,
}
