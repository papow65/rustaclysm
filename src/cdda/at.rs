use crate::prelude::Pos;
use serde::de::Error;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use std::{fmt, marker::PhantomData};

#[derive(Debug, Deserialize)]
pub(crate) struct At<T> {
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) obj: T,
}

impl<T> At<T> {
    pub(crate) const fn get(&self, relative_pos: Pos) -> Option<&T> {
        if relative_pos.x as u8 == self.x && relative_pos.z as u8 == self.y {
            Some(&self.obj)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct AtVec<T>(pub(crate) Vec<At<T>>);

impl<'de, T> Deserialize<'de> for AtVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AtVecVisitor(PhantomData))
    }
}

struct AtVecVisitor<T>(std::marker::PhantomData<T>);

impl<'de, T> Visitor<'de> for AtVecVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = AtVec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence containing [x, y, object, x, y, object, ...]")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result: Vec<At<T>> = Vec::new();
        while let Some(element) = seq.next_element::<serde_json::Value>()? {
            let x = Deserialize::deserialize(element)
                .map_err(|e| A::Error::custom(format!("x error: {e}")))?;
            let element = seq.next_element::<serde_json::Value>()?.unwrap();
            let y = Deserialize::deserialize(element)
                .map_err(|e| A::Error::custom(format!("y error: {e}")))?;
            let element = seq.next_element::<serde_json::Value>()?.unwrap();
            let obj = Deserialize::deserialize(element)
                .map_err(|e| A::Error::custom(format!("obj error: {e}")))?;
            result.push(At { x, y, obj });
        }
        Ok(AtVec(result))
    }
}
