use crate::InfoId;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemAction {
    pub id: InfoId<Self>,
    pub name: ItemActionName,

    #[expect(unused)]
    #[serde(rename = "//")]
    comment: Option<Arc<str>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemActionName {
    pub str: Arc<str>,

    #[serde(rename = "ctxt")]
    pub context: Option<Arc<str>>,
}
