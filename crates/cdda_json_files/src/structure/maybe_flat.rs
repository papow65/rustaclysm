use serde::Deserialize;

/// Used when both a single T, and a list of T can be expected.
#[derive(Clone, Debug, Deserialize)]
#[serde(from = "MaybeFlat<T>")]
pub struct MaybeFlatVec<T>(pub Vec<T>);

impl<T> From<MaybeFlat<T>> for MaybeFlatVec<T> {
    fn from(maybe_flat: MaybeFlat<T>) -> Self {
        Self(match maybe_flat {
            MaybeFlat::Single(single) => vec![single],
            MaybeFlat::Multi(vec) => vec,
        })
    }
}

impl<T> Default for MaybeFlatVec<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
enum MaybeFlat<T> {
    Single(T),
    Multi(Vec<T>),
}
