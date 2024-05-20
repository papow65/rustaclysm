use crate::prelude::ObjectId;
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Recipe {
    pub(crate) result: ObjectId,

    pub(crate) skill_used: Option<String>,

    #[serde(default)]
    pub(crate) difficulty: u8,

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
    List(Vec<(ObjectId, u8)>),
    Other(#[allow(unused)] serde_json::Value),
}

impl Default for BookLearn {
    fn default() -> Self {
        Self::List(Vec::new())
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
#[serde(from = "Vec<RequiredQualityWrap>")]
pub(crate) struct RequiredQualities(pub(crate) Vec<RequiredQuality>);

impl From<Vec<RequiredQualityWrap>> for RequiredQualities {
    fn from(ws: Vec<RequiredQualityWrap>) -> Self {
        Self(
            ws.into_iter()
                .flat_map(|w| match w {
                    RequiredQualityWrap::Single(r) => vec![r],
                    RequiredQualityWrap::List(l) => l,
                })
                .collect(),
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RequiredQualityWrap {
    Single(RequiredQuality),
    List(Vec<RequiredQuality>),
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
