use crate::prelude::*;
use bevy::utils::HashMap;
use serde::de::{Deserializer, Error, SeqAccess, Visitor};
use serde::Deserialize;
use std::{fmt, hash::Hash, marker::PhantomData};

#[derive(Debug)]
pub(crate) struct Repetition<T> {
    pub(crate) obj: T,
    pub(crate) amount: u32,
}

impl<T> TryFrom<serde_json::Value> for Repetition<T>
where
    T: Clone + for<'de> Deserialize<'de>,
{
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        match value {
            serde_json::Value::Array(_) => serde_json::from_value(value),
            _ => serde_json::from_value::<T>(value).map(|obj| Self { obj, amount: 1 }),
        }
    }
}

impl<'de, T> Deserialize<'de> for Repetition<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RepetitionVisitor(PhantomData))
    }
}

struct RepetitionVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for RepetitionVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Repetition<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("item of [item, amount]")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Repetition<T>, E>
    where
        E: Error,
    {
        Ok(Deserialize::deserialize(serde_json::Value::Bool(value))
            .map(|obj| Repetition { obj, amount: 1 })
            .unwrap())
    }

    fn visit_str<E>(self, value: &str) -> Result<Repetition<T>, E>
    where
        E: Error,
    {
        Ok(
            Deserialize::deserialize(serde_json::Value::String(value.to_string()))
                .map(|obj| Repetition { obj, amount: 1 })
                .unwrap(),
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut obj = None;
        let mut amount = None;
        while let Some(element) = seq.next_element::<serde_json::Value>()? {
            assert!(amount == None);
            match obj {
                None => {
                    obj = Some(Deserialize::deserialize(element).unwrap());
                }
                Some(_) => {
                    amount = Some(Deserialize::deserialize(element).unwrap());
                }
            }
        }
        Ok(Repetition {
            obj: obj.unwrap(),
            amount: amount.unwrap(),
        })
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
            let amount = repetition.amount as i32;
            for j in i..i + amount {
                result.insert(
                    location(j.rem_euclid(size), j.div_euclid(size)),
                    &repetition.obj,
                );
            }
            i += amount;
        }
        assert!(i == size * size);
        assert!(result.len() == i as usize, "{} <-> {i}", result.len());
        result
    }
}
