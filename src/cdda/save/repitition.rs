use crate::prelude::{Level, Overzone, Pos, ZoneLevel};
use bevy::utils::HashMap;
use serde::de::Deserializer;
use serde::Deserialize;
use std::hash::Hash;

#[derive(Debug, Deserialize)]
pub(crate) struct Amount<T> {
    pub(crate) obj: T,

    /// can be 1, should not be 0
    pub(crate) amount: u32,
}

#[derive(Debug)]
pub(crate) struct Single<T>(Amount<T>);

impl<'de, T> Deserialize<'de> for Single<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let obj: T = Deserialize::deserialize(deserializer)?;
        Ok(Single(Amount { obj, amount: 1 }))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Repetition<T> {
    Single(Single<T>),
    Multiple(Amount<T>),
}

impl<T> Repetition<T> {
    pub(crate) fn as_amount(&self) -> &Amount<T> {
        match self {
            Self::Single(m) => &m.0,
            Self::Multiple(m) => m,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RepetitionBlock<T>(Vec<Repetition<T>>);

impl<T> RepetitionBlock<T> {
    pub(crate) fn load_as_subzone(&self, subzone_offset: Pos) -> HashMap<Pos, &T> {
        self.load(
            |x, z| {
                subzone_offset
                    .offset(Pos {
                        x,
                        level: Level::ZERO,
                        z,
                    })
                    .unwrap()
            },
            12,
        )
    }

    #[allow(unused)]
    pub(crate) fn load_as_zone_level(&self, zone_level: ZoneLevel) -> HashMap<Pos, &T> {
        let base_pos = zone_level.base_pos();
        self.load(
            |x, z| {
                base_pos
                    .offset(Pos {
                        x,
                        level: Level::ZERO,
                        z,
                    })
                    .unwrap()
            },
            24,
        )
    }

    pub(crate) fn load_as_overzone(
        &self,
        overzone: Overzone,
        level: Level,
    ) -> HashMap<ZoneLevel, &T> {
        self.load(
            |x, z| overzone.base_zone().offset(x, z).zone_level(level),
            180,
        )
    }

    fn load<F, X: Eq + Hash>(&self, location: F, size: i32) -> HashMap<X, &T>
    where
        F: Fn(i32, i32) -> X,
    {
        let mut result = HashMap::new();
        let mut i: i32 = 0;
        for repetition in &self.0 {
            let Amount { obj, amount } = repetition.as_amount();
            let amount = *amount as i32;
            for j in i..i + amount {
                result.insert(location(j.rem_euclid(size), j.div_euclid(size)), obj);
            }
            i += amount;
        }
        assert!(i == size * size);
        assert!(result.len() == i as usize, "{} <-> {i}", result.len());
        result
    }
}
