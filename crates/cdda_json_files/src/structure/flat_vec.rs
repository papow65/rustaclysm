use serde::de::{Deserialize, DeserializeOwned, Deserializer, Error as _, SeqAccess, Visitor};
use std::{fmt, marker::PhantomData};

#[derive(Debug, Default)]
pub struct FlatVec<T, const N: usize>(pub Vec<T>);

impl<'de, T, const N: usize> Deserialize<'de> for FlatVec<T, N>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(FlatSeqVisitor::<T, N>(PhantomData))
            .map(FlatVec)
    }
}

struct FlatSeqVisitor<T, const N: usize>(PhantomData<[T; N]>);

impl<'de, T, const N: usize> Visitor<'de> for FlatSeqVisitor<T, N>
where
    T: DeserializeOwned,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a flattened sequence containing zero or more times the N fields of T")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result: Vec<T> = Vec::new();
        while let Some(element) = seq.next_element::<serde_json::Value>()? {
            let mut fields = [(); N].map(|()| serde_json::Value::default());
            fields[0] = element;
            for field in fields.iter_mut().take(N).skip(1) {
                *field = seq.next_element::<serde_json::Value>()?.ok_or_else(|| {
                    A::Error::custom(String::from("Missing value(s) at the end of the sequence"))
                })?;
            }
            result.push(
                serde_json::from_value(serde_json::Value::Array(Vec::from(fields))).map_err(
                    |e| {
                        A::Error::custom(format!(
                            "FlatSeqVisitor(N={N}) - DeserializeOwned error: {e}"
                        ))
                    },
                )?,
            );
        }
        Ok(result)
    }
}
