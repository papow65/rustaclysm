use crate::gameplay::PosOffset;
use bevy::prelude::Component;
use cdda_json_files::CddaItem;
use std::sync::Arc;

#[derive(Component)]
pub(crate) struct Vehicle;

#[derive(Component)]
pub(crate) struct VehiclePart {
    #[expect(unused)]
    pub(crate) offset: PosOffset,

    #[expect(unused)]
    pub(crate) item: Arc<CddaItem>,
}
