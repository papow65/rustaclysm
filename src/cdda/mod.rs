mod data;
mod save;

pub(crate) use self::{data::*, save::*};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum MaybeFlat<T> {
    Single(T),
    Multi(Vec<T>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "MaybeFlat<T>")]
pub(crate) struct DeflatVec<T>(Vec<T>);

impl<T> From<MaybeFlat<T>> for DeflatVec<T> {
    fn from(maybe_flat: MaybeFlat<T>) -> Self {
        Self(match maybe_flat {
            MaybeFlat::Single(single) => vec![single],
            MaybeFlat::Multi(vec) => vec,
        })
    }
}
