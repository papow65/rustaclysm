use crate::{cdda::CddaItem, gameplay::PosOffset};
use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct Vehicle;

#[derive(Component)]
pub(crate) struct VehiclePart {
    #[allow(unused)]
    pub(crate) offset: PosOffset,

    #[allow(unused)]
    pub(crate) item: CddaItem,
}
