use crate::info::practice::ActivityLevel;
use crate::{
    CommonItemInfo, Ignored, InfoId, Quality, RequiredLinkedLater, Requirement, UntypedInfoId,
};
use bevy_log::error;
use bevy_platform_support::collections::HashMap;
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
    pub activity_level: Option<ActivityLevel>,

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

    pub batch_time_factors: Option<Vec<serde_json::Value>>,
    pub blueprint_excludes: Option<Vec<serde_json::Value>>,
    pub blueprint_name: Option<Arc<str>>,
    pub blueprint_needs: Option<serde_json::Value>,
    pub blueprint_provides: Option<Vec<serde_json::Value>>,
    pub blueprint_requires: Option<Vec<serde_json::Value>>,
    pub blueprint_resources: Option<Vec<serde_json::Value>>,
    pub byproduct_group: Option<Vec<serde_json::Value>>,
    pub byproducts: Option<Vec<serde_json::Value>>,
    pub charges: Option<u16>,
    pub check_blueprint_needs: Option<bool>,
    pub construction_blueprint: Option<Arc<str>>,
    pub contained: Option<bool>,
    pub container: Option<Arc<str>>,
    pub decomp_learn: Option<u8>,
    pub delete_flags: Option<Vec<serde_json::Value>>,
    pub description: Option<Arc<str>>,
    pub extend: Option<serde_json::Value>,
    pub flags: Option<Vec<serde_json::Value>>,
    pub never_learn: Option<bool>,
    pub proficiencies: Option<Vec<serde_json::Value>>,
    pub result_mult: Option<u8>,
    pub reversible: Option<serde_json::Value>,
    pub skills_required: Option<Vec<serde_json::Value>>,
    pub subcategory: Option<Arc<str>>,
    pub tools: Option<Vec<serde_json::Value>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
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

#[derive(Debug, Deserialize)]
#[serde(from = "CddaAlternative")]
pub enum Alternative {
    Item {
        item: RequiredLinkedLater<CommonItemInfo>,
        required: u32,
        recoverable: bool,
    },
    Requirement {
        requirement: RequiredLinkedLater<Requirement>,
        factor: u32,
    },
}

impl From<CddaAlternative> for Alternative {
    fn from(source: CddaAlternative) -> Self {
        match source {
            CddaAlternative::Item(item, required) => Self::Item {
                item: RequiredLinkedLater::from(item),
                required,
                recoverable: true,
            },
            CddaAlternative::Dynamic(requirement, factor, note) => match &*note {
                "LIST" => Self::Requirement {
                    requirement: RequiredLinkedLater::from(this::<Requirement>(requirement.into())),
                    factor,
                },
                "NO_RECOVER" => Self::Item {
                    item: RequiredLinkedLater::from(this::<CommonItemInfo>(requirement.into())),
                    required: factor,
                    recoverable: false,
                },
                unexpected => {
                    error!("Unexpected alternative tag {unexpected}");
                    Self::Requirement {
                        requirement: RequiredLinkedLater::from(this::<Requirement>(
                            requirement.into(),
                        )),
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
    pub requirement: RequiredLinkedLater<Requirement>,
    pub factor: u32,
    pub kind: UsingKind,
}

impl Using {
    pub fn to_components(
        &self,
        called_from: impl AsRef<str> + Clone,
    ) -> Option<Vec<Vec<Alternative>>> {
        let requirement = self.requirement.get_option(called_from.clone())?;

        Some(if self.kind == UsingKind::Components {
            requirement
                .components
                .iter()
                .map(|component| {
                    component
                        .iter()
                        .filter_map(|alternative| match alternative {
                            Alternative::Item {
                                item,
                                required,
                                recoverable,
                            } => {
                                let item = item.get_option(called_from.clone())?;
                                Some(Alternative::Item {
                                    item: RequiredLinkedLater::new_final(item.id.clone(), &item),
                                    required: required * self.factor,
                                    recoverable: *recoverable,
                                })
                            }
                            Alternative::Requirement {
                                requirement,
                                factor,
                            } => {
                                let requirement = requirement.get_option(called_from.clone())?;
                                Some(Alternative::Requirement {
                                    requirement: RequiredLinkedLater::new_final(
                                        requirement.id.clone(),
                                        &requirement,
                                    ),
                                    factor: factor * self.factor,
                                })
                            }
                        })
                        .collect()
                })
                .collect()
        } else {
            vec![vec![Alternative::Requirement {
                requirement: RequiredLinkedLater::new_final(requirement.id.clone(), &requirement),
                factor: self.factor,
            }]]
        })
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
            CddaUsing::List(requirement, factor, kind) => {
                if &*kind != "LIST" {
                    error!("Unexpected value for using kind: {kind:?}");
                }

                Self {
                    requirement,
                    factor,
                    kind: UsingKind::Alternatives,
                }
            }
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
    NonList(RequiredLinkedLater<Requirement>, u32),
    List(RequiredLinkedLater<Requirement>, u32, Arc<str>),
}

const fn this<T>(info_id: InfoId<T>) -> InfoId<T> {
    info_id
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
