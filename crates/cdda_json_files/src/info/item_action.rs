use crate::{Ignored, InfoId};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ItemAction {
    pub id: InfoId<Self>,
    pub name: ItemActionName,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemActionName {
    pub str: Arc<str>,

    #[serde(rename = "ctxt")]
    pub context: Option<Arc<str>>,
}
