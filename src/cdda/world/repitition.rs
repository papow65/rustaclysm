use crate::prelude::{
    Level, LevelOffset, ObjectId, Overzone, Pos, PosOffset, SubzoneLevel, ZoneLevel,
};
use bevy::utils::HashMap;
use serde::de::{Deserializer, Error};
use serde::Deserialize;
use std::hash::Hash;

#[derive(Debug, Deserialize)]
pub(crate) struct CddaAmount<T> {
    pub(crate) obj: T,

    /// can be 1, should not be 0
    pub(crate) amount: u32,
}

#[derive(Debug)]
pub(crate) struct Single<T>(CddaAmount<T>);

impl<'de, T> Deserialize<'de> for Single<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let obj: T = Deserialize::deserialize(deserializer)?;
        Ok(Single(CddaAmount { obj, amount: 1 }))
    }
}

#[derive(Debug)]
pub(crate) enum Repetition<T> {
    Single(Single<T>),
    Multiple(CddaAmount<T>),
}

impl<T> Repetition<T> {
    pub(crate) const fn as_amount(&self) -> &CddaAmount<T> {
        match self {
            Self::Single(m) => &m.0,
            Self::Multiple(m) => m,
        }
    }
}

impl<'de, T> Deserialize<'de> for Repetition<T>
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match Single::deserialize(value.clone()) {
            Ok(single) => Ok(Repetition::Single(single)),
            Err(single_error) => match CddaAmount::deserialize(value) {
                Ok(amount) => Ok(Repetition::Multiple(amount)),
                Err(amount_error) => {
                    eprintln!("{single_error:?}");
                    eprintln!("{amount_error:?}");
                    Err(amount_error).map_err(D::Error::custom)
                }
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RepetitionBlock<T>(Vec<Repetition<T>>);

impl<T> RepetitionBlock<T> {
    pub(crate) fn new(amount: CddaAmount<T>) -> Self {
        Self(vec![Repetition::Multiple(amount)])
    }

    pub(crate) fn load_as_subzone(&self, subzone_level: SubzoneLevel) -> HashMap<Pos, &T> {
        self.load(
            |x, z| {
                subzone_level
                    .base_pos()
                    .offset(PosOffset {
                        x,
                        level: LevelOffset::ZERO,
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
                    .offset(PosOffset {
                        x,
                        level: LevelOffset::ZERO,
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
            let CddaAmount { obj, amount } = repetition.as_amount();
            let amount = *amount as i32;
            for j in i..i + amount {
                result.insert(location(j.rem_euclid(size), j.div_euclid(size)), obj);
            }
            i += amount;
        }
        assert_eq!(i, size * size);
        assert_eq!(result.len(), i as usize);
        result
    }
}

impl RepetitionBlock<ObjectId> {
    pub(crate) fn is_significant(&self) -> bool {
        1 < self.0.len()
            || ![
                ObjectId::new("t_open_air"),
                ObjectId::new("t_soil"),
                ObjectId::new("t_rock"),
            ]
            .contains(&self.0.first().unwrap().as_amount().obj)
    }
}
