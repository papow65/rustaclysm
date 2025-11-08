use crate::info::practice::ActivityLevel;
use crate::{
    CommonItemInfo, Flags, Ignored, InfoId, Quality, RequiredComponent, RequiredLinkedLater,
    RequiredPart, RequiredTool, Requirement, UntypedInfoId,
};
use bevy_log::error;
use bevy_platform::collections::HashMap;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::hash::{Hash, Hasher};
use std::{num::NonZeroU32, ops::Mul, sync::Arc};
use units::Duration;

// PartialEq, Eq, and Hash manually implemented below
#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub id: InfoId<Self>,

    #[serde(flatten)]
    pub result: RecipeResult,

    pub subcategory: Arc<str>,

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
    pub components: Vec<Vec<Alternative<RequiredComponent>>>,

    #[serde(default)]
    pub using: Vec<Using>,

    pub batch_time_factors: Option<Vec<JsonValue>>,
    pub blueprint_excludes: Option<Vec<JsonValue>>,
    pub blueprint_name: Option<Arc<str>>,
    pub blueprint_needs: Option<JsonValue>,
    pub blueprint_provides: Option<Vec<JsonValue>>,
    pub blueprint_requires: Option<Vec<JsonValue>>,
    pub blueprint_resources: Option<Vec<JsonValue>>,
    pub byproduct_group: Option<Vec<JsonValue>>,
    pub byproducts: Option<Vec<JsonValue>>,
    pub charges: Option<u16>,
    pub check_blueprint_needs: Option<bool>,
    pub construction_blueprint: Option<Arc<str>>,
    pub contained: Option<bool>,
    pub container: Option<Arc<str>>,
    pub decomp_learn: Option<u8>,
    pub delete_flags: Option<Vec<JsonValue>>,
    pub extend: Option<JsonValue>,
    pub flags: Flags,
    pub never_learn: Option<bool>,
    pub proficiencies: Option<Vec<JsonValue>>,
    pub result_mult: Option<u8>,
    pub reversible: Option<JsonValue>,
    pub skills_required: Option<Vec<JsonValue>>,

    #[serde(default)]
    pub tools: Vec<Vec<Alternative<RequiredTool>>>,

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
#[serde(tag = "category")]
pub enum RecipeResult {
    #[serde(alias = "CC_BUILDING")]
    Camp {
        result: UntypedInfoId,
        description: Arc<str>,
    },

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
    Item {
        #[serde(rename = "result")]
        item_info: RequiredLinkedLater<CommonItemInfo>,
    },

    #[serde(untagged)]
    Unknown { result: UntypedInfoId },
}

impl RecipeResult {
    #[track_caller]
    pub fn item_info(&self) -> Option<Arc<CommonItemInfo>> {
        if let Self::Item { item_info } = self {
            item_info.get_option()
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BookLearn {
    List(Vec<BookLearnItem>),
    Map(HashMap<InfoId<CommonItemInfo>, JsonValue>),
    Other(JsonValue),
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

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct RequiredQuality {
    #[serde(rename = "id")]
    pub quality: RequiredLinkedLater<Quality>,
    pub level: u8,
}

#[derive(Debug, Deserialize)]
#[serde(from = "CddaAlternative<R>")]
pub enum Alternative<R: RequiredPart> {
    Item {
        item: RequiredLinkedLater<CommonItemInfo>,
        required: R,
        recoverable: bool,
    },
    Requirement {
        requirement: RequiredLinkedLater<Requirement>,
        factor: R,
    },
}

impl<R: RequiredPart> Mul<R> for &Alternative<R> {
    type Output = Alternative<R>;

    fn mul(self, rhs: R) -> Self::Output {
        match self {
            Alternative::Item {
                item,
                required,
                recoverable,
            } => Alternative::Item {
                item: item.clone(),
                required: *required * rhs,
                recoverable: *recoverable,
            },
            Alternative::Requirement {
                requirement,
                factor,
            } => Alternative::Requirement {
                requirement: requirement.clone(),
                factor: *factor * rhs,
            },
        }
    }
}

impl<R: RequiredPart> From<CddaAlternative<R>> for Alternative<R> {
    fn from(source: CddaAlternative<R>) -> Self {
        match source {
            CddaAlternative::Item(item) => Self::Item {
                item: RequiredLinkedLater::from(item),
                required: R::present(),
                recoverable: true,
            },
            CddaAlternative::ItemAmount(item, required) => Self::Item {
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
pub enum CddaAlternative<R> {
    Item(InfoId<CommonItemInfo>),
    ItemAmount(InfoId<CommonItemInfo>, R),
    Dynamic(UntypedInfoId, R, Arc<str>),
}

#[derive(Debug, Deserialize)]
#[serde(from = "CddaUsing")]
pub struct Using {
    pub requirement: RequiredLinkedLater<Requirement>,
    pub factor: NonZeroU32,

    // Barely used
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
    NonList(RequiredLinkedLater<Requirement>, NonZeroU32),
    List(RequiredLinkedLater<Requirement>, NonZeroU32, Arc<str>),
}

const fn this<T>(info_id: InfoId<T>) -> InfoId<T> {
    info_id
}

#[cfg(test)]
mod recipe_tests {
    use super::*;
    use serde_json::from_str as from_json_str;

    #[test]
    fn hammer_works() {
        let json = include_str!("test_data/hammer.json");
        let recipe = from_json_str::<Recipe>(json);
        let recipe = recipe.as_ref();
        assert!(
            recipe.is_ok_and(|recipe| matches!(recipe.result, RecipeResult::Item { .. })),
            "{:?}",
            recipe.map(|recipe| &recipe.result)
        );
    }

    #[test]
    fn building_works() {
        let json = include_str!("test_data/building.json");
        let recipe = from_json_str::<Recipe>(json);
        let recipe = recipe.as_ref();
        assert!(
            recipe.is_ok_and(|recipe| matches!(recipe.result, RecipeResult::Camp { .. })),
            "{:?}",
            recipe.map(|recipe| &recipe.result)
        );
    }
}
