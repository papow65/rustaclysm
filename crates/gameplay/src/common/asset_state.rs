#[derive(Debug)]
pub(crate) enum AssetState<'a, T> {
    Available { asset: &'a T },
    Loading,
    Nonexistent,
}
