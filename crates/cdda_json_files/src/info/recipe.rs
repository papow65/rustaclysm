use crate::{
    CommonItemInfo, HashMap, InfoId, Quality, RequiredLinkedLater, Requirement, UntypedInfoId,
};
use bevy_log::error;
use serde::Deserialize;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use units::Duration;

// PartialEq, Eq, and Hash manually implemented below
#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub id: InfoId<Self>,

    #[serde(flatten)]
    pub result: RecipeResult,

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

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Recipe {}

impl Hash for Recipe {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "category", content = "result")]
pub enum RecipeResult {
    #[serde(alias = "CC_BUILDING")]
    Camp(UntypedInfoId),

    #[serde(alias = "CC_*")]
    #[serde(alias = "CC_AMMO")]
    #[serde(alias = "CC_ANIMALS")]
    #[serde(alias = "CC_APPLIANCE")]
    #[serde(alias = "CC_ARMOR")]
    #[serde(alias = "CC_CHEM")]
    #[serde(alias = "CC_ELECTRONIC")]
    #[serde(alias = "CC_FOOD")]
    #[serde(alias = "CC_OTHER")]
    #[serde(alias = "CC_WEAPON")]
    Item(RequiredLinkedLater<CommonItemInfo>),

    #[serde(untagged)]
    Unknown(UntypedInfoId),
}

impl RecipeResult {
    pub fn item_info(&self, source: impl AsRef<str>) -> Option<Arc<CommonItemInfo>> {
        if let Self::Item(info) = self {
            info.get_option(source)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BookLearn {
    List(Vec<BookLearnItem>),
    Map(HashMap<InfoId<CommonItemInfo>, serde_json::Value>),
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
    Simple(InfoId<CommonItemInfo>),
    Wrapped((InfoId<CommonItemInfo>,)),
    WithSkill(InfoId<CommonItemInfo>, u8),
}

impl BookLearnItem {
    #[must_use]
    pub fn id(&self) -> InfoId<CommonItemInfo> {
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
    #[serde(rename = "id")]
    pub quality: RequiredLinkedLater<Quality>,
    pub level: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "CddaAlternative")]
pub enum Alternative {
    Item {
        item: InfoId<CommonItemInfo>,
        required: u32,
        recoverable: bool,
    },
    Requirement {
        requirement: InfoId<Requirement>,
        factor: u32,
    },
}

impl From<CddaAlternative> for Alternative {
    fn from(source: CddaAlternative) -> Self {
        match source {
            CddaAlternative::Item(item, required) => Self::Item {
                item,
                required,
                recoverable: true,
            },
            CddaAlternative::Dynamic(requirement, factor, note) => match &*note {
                "LIST" => Self::Requirement {
                    requirement: requirement.into(),
                    factor,
                },
                "NO_RECOVER" => Self::Item {
                    item: requirement.into(),
                    required: factor,
                    recoverable: false,
                },
                unexpected => {
                    error!("Unexpected alternative tag {unexpected}");
                    Self::Requirement {
                        requirement: requirement.into(),
                        factor,
                    }
                }
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CddaAlternative {
    Item(InfoId<CommonItemInfo>, u32),
    Dynamic(UntypedInfoId, u32, Arc<str>),
}

#[derive(Debug, Deserialize)]
#[serde(from = "CddaUsing")]
pub struct Using {
    pub requirement: InfoId<Requirement>,
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
    NonList(InfoId<Requirement>, u32),
    List(InfoId<Requirement>, u32, Arc<str>),
}

#[cfg(test)]
mod recipe_tests {
    use super::*;

    #[test]
    fn hammer_works() {
        let json = include_str!("test_hammer.json");
        let recipe = serde_json::from_str::<Recipe>(json);
        let recipe = recipe.as_ref();
        assert!(
            recipe.is_ok_and(|recipe| matches!(recipe.result, RecipeResult::Item(_))),
            "{:?}",
            recipe.map(|recipe| &recipe.result)
        );
    }

    #[test]
    fn building_works() {
        let json = include_str!("test_building.json");
        let recipe = serde_json::from_str::<Recipe>(json);
        let recipe = recipe.as_ref();
        assert!(
            recipe.is_ok_and(|recipe| matches!(recipe.result, RecipeResult::Camp(_))),
            "{:?}",
            recipe.map(|recipe| &recipe.result)
        );
    }
}
