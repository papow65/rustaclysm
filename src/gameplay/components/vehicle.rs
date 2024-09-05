use crate::{cdda::CddaItem, gameplay::PosOffset};
use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct Vehicle;

#[derive(Component)]
pub(crate) struct VehiclePart {
    #[expect(unused)]
    pub(crate) offset: PosOffset,

    #[expect(unused)]
    pub(crate) item: CddaItem,
}
