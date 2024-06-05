use crate::prelude::ObjectId;
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Recipe {
    pub(crate) result: ObjectId,

    pub(crate) skill_used: Option<String>,

    #[serde(default)]
    pub(crate) difficulty: u8,

    pub(crate) time: Option<String>,

    #[serde(default)]
    pub(crate) book_learn: BookLearn,

    #[serde(default)]
    pub(crate) autolearn: AutoLearn,

    #[serde(default)]
    pub(crate) qualities: RequiredQualities,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum BookLearn {
    List(Vec<BookLearnItem>),
    Map(HashMap<ObjectId, serde_json::Value>),
    Other(#[allow(unused)] serde_json::Value),
}

impl Default for BookLearn {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum BookLearnItem {
    Simple(ObjectId),
    Wrapped((ObjectId,)),
    WithSkill(ObjectId, #[allow(unused)] u8),
}

impl BookLearnItem {
    pub(crate) const fn id(&self) -> &ObjectId {
        match self {
            Self::Simple(id) | Self::Wrapped((id,)) | Self::WithSkill(id, _) => id,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum AutoLearn {
    Bool(bool),
    Skills(Vec<(String, u8)>),
}

impl Default for AutoLearn {
    fn default() -> Self {
        Self::Bool(false)
    }
}

#[derive(Default, Debug, Deserialize)]
#[serde(from = "Vec<Wrap<RequiredQuality>>")]
pub(crate) struct RequiredQualities(pub(crate) Vec<RequiredQuality>);

impl From<Vec<Wrap<RequiredQuality>>> for RequiredQualities {
    fn from(ws: Vec<Wrap<RequiredQuality>>) -> Self {
        Self(
            ws.into_iter()
                .flat_map(|w| match w {
                    Wrap::Single(r) => vec![r],
                    Wrap::List(l) => l,
                })
                .collect(),
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Wrap<T> {
    Single(T),
    List(Vec<T>),
}

#[derive(Debug, Deserialize)]
pub(crate) struct RequiredQuality {
    pub(crate) id: ObjectId,
    pub(crate) level: u8,
}

#[cfg(test)]
mod character_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_hammer.json");
        let result = serde_json::from_str::<Recipe>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
