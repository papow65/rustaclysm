use bevy::prelude::Component;
use cdda_json_files::CddaItem;
use gameplay_location::PosOffset;
use std::sync::Arc;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct Vehicle;

#[derive(Component)]
#[component(immutable)]
pub(crate) struct VehiclePart {
    #[expect(unused)]
    pub(crate) offset: PosOffset,

    #[expect(unused)]
    pub(crate) item: Arc<CddaItem>,
}
