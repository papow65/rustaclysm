use crate::{
    Alternative, AutoLearn, BookLearn, CommonItemInfo, Ignored, InfoId, RequiredQualities,
    UntypedInfoId, Using,
};
use serde::Deserialize;
use std::sync::Arc;
use units::Duration;

#[derive(Debug, Deserialize)]
pub struct Practice {
    pub id: InfoId<Self>,

    pub activity_level: ActivityLevel,
    pub category: Arc<str>,
    pub subcategory: Arc<str>,
    pub name: Arc<str>,
    pub description: Arc<str>,
    pub skill_used: Arc<str>,
    pub time: Duration,

    #[serde(default)]
    pub tools: Vec<serde_json::Value>,

    #[serde(default)]
    pub components: Vec<Vec<Alternative>>,

    #[serde(default)]
    pub autolearn: AutoLearn,

    #[serde(default)]
    pub book_learn: BookLearn,

    #[serde(default)]
    pub flags: Vec<Arc<str>>,

    #[serde(default)]
    pub skills_required: Vec<(Arc<str>, u8)>,

    #[serde(default)]
    pub using: Vec<Using>,

    #[serde(alias = "byproducts")]
    #[serde(default)]
    pub byproduct_group: Vec<Byproduct>,

    #[serde(default)]
    pub qualities: RequiredQualities,

    pub practice_data: PracticeData,

    #[serde(default)]
    pub proficiencies: Vec<RequiredProficiency>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub enum ActivityLevel {
    #[serde(rename = "NO_EXERCISE")]
    None,

    #[serde(rename = "LIGHT_EXERCISE")]
    Light,

    #[serde(rename = "MODERATE_EXERCISE")]
    Moderate,

    #[serde(rename = "ACTIVE_EXERCISE")]
    Active,

    #[serde(rename = "BRISK_EXERCISE")]
    Brisk,

    #[serde(rename = "EXTRA_EXERCISE")]
    Extra,

    #[serde(rename = "fake")]
    Fake,
}

#[derive(Debug, Deserialize)]
pub struct Byproduct {
    pub item: InfoId<CommonItemInfo>,
    pub charges: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PracticeData {
    pub min_difficulty: u8,
    pub max_difficulty: u8,
    pub skill_limit: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RequiredProficiency {
    Hard {
        proficiency: UntypedInfoId,
        required: bool,
    },
    Soft {
        proficiency: UntypedInfoId,
        fail_multiplier: u8,
        time_multiplier: f32,
    },
}
