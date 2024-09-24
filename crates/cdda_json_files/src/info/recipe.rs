use crate::ObjectId;
use bevy::utils::HashMap;
use serde::Deserialize;
use std::sync::Arc;
use units::Duration;

#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub result: ObjectId,

    pub skill_used: Option<Arc<str>>,

    #[serde(default)]
    pub difficulty: u8,

    pub time: Option<Duration>,

    #[serde(default)]
    pub book_learn: BookLearn,

    #[serde(default)]
    pub autolearn: AutoLearn,

    #[serde(default)]
    pub qualities: RequiredQualities,

    #[serde(default)]
    pub components: Vec<Vec<Alternative>>,

    #[serde(default)]
    pub using: Vec<Using>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BookLearn {
    List(Vec<BookLearnItem>),
    Map(HashMap<ObjectId, serde_json::Value>),
    Other(serde_json::Value),
}

impl Default for BookLearn {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BookLearnItem {
    Simple(ObjectId),
    Wrapped((ObjectId,)),
    WithSkill(ObjectId, u8),
}

impl BookLearnItem {
    pub fn id(&self) -> ObjectId {
        match self {
            Self::Simple(id) | Self::Wrapped((id,)) | Self::WithSkill(id, _) => id.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AutoLearn {
    Bool(bool),
    Skills(Vec<(Arc<str>, u8)>),
}

impl Default for AutoLearn {
    fn default() -> Self {
        Self::Bool(false)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(from = "Vec<Wrap<RequiredQuality>>")]
pub struct RequiredQualities(pub Vec<RequiredQuality>);

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
pub enum Wrap<T> {
    Single(T),
    List(Vec<T>),
}

#[derive(Debug, Deserialize)]
pub struct RequiredQuality {
    pub id: ObjectId,
    pub level: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "CddaAlternative")]
pub enum Alternative {
    Item { item: ObjectId, required: u32 },
    Requirement { requirement: ObjectId, factor: u32 },
}

impl From<CddaAlternative> for Alternative {
    fn from(source: CddaAlternative) -> Self {
        match source {
            CddaAlternative::Item(item, required) => Self::Item { item, required },
            CddaAlternative::List(requirement, factor, _) => Self::Requirement {
                requirement,
                factor,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CddaAlternative {
    Item(ObjectId, u32),
    List(ObjectId, u32, Arc<str>),
}

#[derive(Debug, Deserialize)]
#[serde(from = "CddaUsing")]
pub struct Using {
    pub requirement: ObjectId,
    pub factor: u32,
    pub kind: UsingKind,
}

impl From<CddaUsing> for Using {
    fn from(source: CddaUsing) -> Self {
        match source {
            CddaUsing::NonList(requirement, factor) => Self {
                requirement,
                factor,
                kind: UsingKind::Components,
            },
            CddaUsing::List(requirement, factor, _) => Self {
                requirement,
                factor,
                kind: UsingKind::Alternatives,
            },
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum UsingKind {
    Components,
    Alternatives,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CddaUsing {
    NonList(ObjectId, u32),
    List(ObjectId, u32, Arc<str>),
}

#[cfg(test)]
mod recipe_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_hammer.json");
        let result = serde_json::from_str::<Recipe>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
