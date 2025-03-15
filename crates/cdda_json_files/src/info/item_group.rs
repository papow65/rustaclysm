use crate::{BashItem, InfoId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ItemGroup {
    pub subtype: Option<InfoId>,

    pub items: Option<Vec<BashItem>>,
    pub entries: Option<Vec<BashItem>>,
}
