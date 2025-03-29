use crate::{
    AutoLearn, BookLearn, CommonItemInfo, InfoId, RequiredQualities, UntypedInfoId, Using,
};
use bevy_platform_support::collections::HashMap;
use serde::Deserialize;
use std::sync::Arc;
use units::Duration;

#[derive(Debug, Deserialize)]
pub struct Practice {
    pub id: InfoId<Self>,

    pub activty_level: Arc<str>,
    pub category: Arc<str>,
    pub subcategory: Arc<str>,
    pub name: Arc<str>,
    pub description: Arc<str>,
    pub skill_used: Arc<str>,
    pub time: Duration,
    pub tools: Arc<str>,
    pub components: Arc<str>,
    pub autolearn: AutoLearn,
    pub book_learn: BookLearn,
    pub flags: Arc<str>,
    pub skills_required: Arc<str>,
    pub using: Vec<Using>,

    #[serde(default)]
    pub byproduct_group: Option<ByproductGroup>,

    #[serde(default)]
    pub qualities: RequiredQualities,

    pub practice_data: PracticeData,

    #[serde(default)]
    pub proficiencies: Vec<RequiredProficiency>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ByproductGroup {
    pub item: InfoId<CommonItemInfo>,
    pub charges: u8,
}

#[derive(Debug, Deserialize)]
pub struct PracticeData {
    pub min_difficulty: u8,
    pub max_difficulty: u8,
    pub skill_limit: u8,
}

#[derive(Debug, Deserialize)]
pub enum RequiredProficiency {
    Hard {
        proficiency: UntypedInfoId,
    },
    Soft {
        proficiency: UntypedInfoId,
        fail_multiplier: u8,
        time_difficulty: u8,
    },
}
