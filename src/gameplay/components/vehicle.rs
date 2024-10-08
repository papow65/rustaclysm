use crate::gameplay::PosOffset;
use bevy::prelude::Component;
use cdda_json_files::CddaItem;

#[derive(Component)]
pub(crate) struct Vehicle;

#[derive(Component)]
pub(crate) struct VehiclePart {
    #[expect(unused)]
    pub(crate) offset: PosOffset,

    #[expect(unused)]
    pub(crate) item: CddaItem,
}
