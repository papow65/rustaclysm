use crate::gameplay::{Infos, RelativeSegments, TileLoader};
use bevy::ecs::system::SystemParam;
use bevy::prelude::Res;

#[derive(SystemParam)]
pub(crate) struct GameplayReadiness<'w> {
    tile_loader: Option<Res<'w, TileLoader>>,
    infos: Option<Res<'w, Infos>>,
    relative_segments: Option<Res<'w, RelativeSegments>>,
}

impl<'w> GameplayReadiness<'w> {
    pub(crate) const fn ready_to_load(&self) -> bool {
        self.tile_loader.is_some() && self.infos.is_some() && self.relative_segments.is_some()
    }
}
