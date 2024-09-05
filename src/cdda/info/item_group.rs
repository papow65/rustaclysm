use crate::{cdda::BashItem, gameplay::ObjectId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ItemGroup {
    #[expect(unused)]
    pub(crate) subtype: Option<ObjectId>,
    pub(crate) items: Option<Vec<BashItem>>,
    pub(crate) entries: Option<Vec<BashItem>>,
}
