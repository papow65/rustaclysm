use crate::prelude::{BashItem, ObjectId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ItemGroup {
    #[allow(unused)]
    pub(crate) subtype: Option<ObjectId>,
    pub(crate) items: Option<Vec<BashItem>>,
    pub(crate) entries: Option<Vec<BashItem>>,
}
