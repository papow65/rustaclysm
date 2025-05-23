use crate::{RequiredLinkedLater, TerrainInfo};
use bevy_log::error;
use serde::Deserialize;
use serde::de::{Deserializer, Error as _};

#[derive(Debug, Deserialize)]
pub struct CddaAmount<T> {
    pub obj: T,

    /// can be 1, should not be 0
    pub amount: u32,
}

#[derive(Debug)]
pub struct Single<T>(CddaAmount<T>);

impl<'de, T> Deserialize<'de> for Single<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let obj: T = Deserialize::deserialize(deserializer)?;
        Ok(Self(CddaAmount { obj, amount: 1 }))
    }
}

#[derive(Debug)]
pub enum Repetition<T> {
    Single(Single<T>),
    Multiple(CddaAmount<T>),
}

impl<T> Repetition<T> {
    pub const fn as_amount(&self) -> &CddaAmount<T> {
        match self {
            Self::Single(m) => &m.0,
            Self::Multiple(m) => m,
        }
    }
}

impl Repetition<RequiredLinkedLater<TerrainInfo>> {
    fn is_significant(&self) -> bool {
        if let Self::Multiple(amount) = self {
            amount.obj.is_significant()
        } else {
            true
        }
    }
}

impl<'de, T> Deserialize<'de> for Repetition<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match Single::deserialize(value.clone()) {
            Ok(single) => Ok(Self::Single(single)),
            Err(single_error) => match CddaAmount::deserialize(value) {
                Ok(amount) => Ok(Self::Multiple(amount)),
                Err(amount_error) => {
                    error!("{single_error:?}");
                    error!("{amount_error:?}");
                    Err(D::Error::custom(amount_error))
                }
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RepetitionBlock<T>(pub Vec<Repetition<T>>);

impl<T> RepetitionBlock<T> {
    pub fn new(amount: CddaAmount<T>) -> Self {
        Self(vec![Repetition::Multiple(amount)])
    }
}

impl RepetitionBlock<RequiredLinkedLater<TerrainInfo>> {
    #[must_use]
    pub fn is_significant(&self) -> bool {
        1 < self.0.len() || self.0.first().expect("Non-empty list").is_significant()
    }
}
