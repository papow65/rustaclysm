mod error;
mod info;
mod plugin;
mod tile;
mod world;

pub(crate) use self::{error::Error, info::*, plugin::CddaPlugin, tile::*, world::*};
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
pub(crate) struct DeflatVec<T>(#[allow(dead_code)] Vec<T>);

impl<T> From<MaybeFlat<T>> for DeflatVec<T> {
    fn from(maybe_flat: MaybeFlat<T>) -> Self {
        Self(match maybe_flat {
            MaybeFlat::Single(single) => vec![single],
            MaybeFlat::Multi(vec) => vec,
        })
    }
}
