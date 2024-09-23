use crate::{BashItem, ObjectId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ItemGroup {
    pub subtype: Option<ObjectId>,

    pub items: Option<Vec<BashItem>>,
    pub entries: Option<Vec<BashItem>>,
}
