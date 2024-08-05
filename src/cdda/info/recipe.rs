use crate::gameplay::{Duration, Infos, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Recipe {
    pub(crate) result: ObjectId,

    pub(crate) skill_used: Option<String>,

    #[serde(default)]
    pub(crate) difficulty: u8,

    pub(crate) time: Option<Duration>,

    #[serde(default)]
    pub(crate) book_learn: BookLearn,

    #[serde(default)]
    pub(crate) autolearn: AutoLearn,

    #[serde(default)]
    pub(crate) qualities: RequiredQualities,

    #[serde(default)]
    pub(crate) components: Vec<Vec<Alternative>>,

    #[serde(default)]
    pub(crate) using: Vec<Using>,

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

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "CddaAlternative")]
pub(crate) enum Alternative {
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
pub(crate) enum CddaAlternative {
    Item(ObjectId, u32),
    List(ObjectId, u32, #[allow(unused)] String),
}

#[derive(Debug, Deserialize)]
#[serde(from = "CddaUsing")]
pub(crate) struct Using {
    pub(crate) requirement: ObjectId,
    pub(crate) factor: u32,
    pub(crate) kind: UsingKind,
}

impl Using {
    pub(crate) fn to_components(&self, infos: &Infos) -> Vec<Vec<Alternative>> {
        if self.kind == UsingKind::Components {
            infos
                .requirement(&self.requirement)
                .components
                .clone()
                .into_iter()
                .map(|component| {
                    component
                        .into_iter()
                        .map(|mut alternative| {
                            *match alternative {
                                Alternative::Item {
                                    ref mut required, ..
                                } => required,
                                Alternative::Requirement { ref mut factor, .. } => factor,
                            } *= self.factor;
                            alternative
                        })
                        .collect()
                })
                .collect()
        } else {
            vec![vec![Alternative::Requirement {
                requirement: self.requirement.clone(),
                factor: self.factor,
            }]]
        }
    }
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
pub(crate) enum UsingKind {
    Components,
    Alternatives,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum CddaUsing {
    NonList(ObjectId, u32),
    List(ObjectId, u32, #[allow(unused)] String),
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
