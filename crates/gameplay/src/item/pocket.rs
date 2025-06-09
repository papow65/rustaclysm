use crate::{Fragment, PocketContents, PocketOf, Shared};
use bevy::ecs::query::QueryData;
use bevy::prelude::{Component, Entity};
use cdda_json_files::{CddaPocket, PocketInfo, SealedData};

#[derive(Copy, Clone, Debug, Component)]
#[component(immutable)]
pub(crate) struct SealedPocket;

impl SealedPocket {
    #[expect(clippy::unused_self)]
    pub(crate) fn suffix(self) -> Fragment {
        Fragment::good("sealed")
    }
}

impl TryFrom<&CddaPocket> for SealedPocket {
    type Error = ();

    fn try_from(pocket: &CddaPocket) -> Result<Self, ()> {
        pocket.sealed.then_some(Self).ok_or(())
    }
}

impl From<&SealedData> for SealedPocket {
    fn from(_: &SealedData) -> Self {
        Self
    }
}

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub(crate) struct Pocket {
    pub(crate) entity: Entity,
    pub(crate) sealed: Option<&'static SealedPocket>,
    pub(crate) info: &'static Shared<PocketInfo>,
    pub(crate) pocket_of: Option<&'static PocketOf>, // TODO
    pub(crate) contents: Option<&'static PocketContents>,
}
