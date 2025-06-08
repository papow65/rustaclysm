use crate::{Explored, RelativeSegments, SubzoneLevelEntities};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Assets, Res, debug};
use gameplay_cdda::{
    Infos, MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset, TileLoader,
};

#[derive(SystemParam)]
pub struct GameplayReadiness<'w> {
    tile_loader: Option<Res<'w, TileLoader>>,
    infos: Option<Res<'w, Infos>>,
    relative_segments: Option<Res<'w, RelativeSegments>>,
    overmap_assets: Res<'w, Assets<OvermapAsset>>,
    overmap_buffer_assets: Res<'w, Assets<OvermapBufferAsset>>,
    map_assets: Res<'w, Assets<MapAsset>>,
    map_memory_assets: Res<'w, Assets<MapMemoryAsset>>,
    explored: Option<Res<'w, Explored>>,
    subzone_level_entities: Option<Res<'w, SubzoneLevelEntities>>,
}

impl GameplayReadiness<'_> {
    #[must_use]
    pub const fn ready_to_load(&self) -> bool {
        self.tile_loader.is_some() && self.infos.is_some() && self.relative_segments.is_some()
    }

    pub(crate) fn ready_to_run(&self) -> bool {
        debug!(
            "Readiness: {} overmaps, {} overmap bufers, {} maps, {} map memories, explored {}, and subzone levels {}",
            self.overmap_assets.len(),
            self.overmap_buffer_assets.len(),
            self.map_assets.len(),
            self.map_memory_assets.len(),
            match &self.explored {
                None => "missing",
                Some(explored) if explored.loaded() => "loaded",
                Some(_) => "loading",
            },
            match &self.subzone_level_entities {
                None => "missing",
                Some(subzone_level_entities) if subzone_level_entities.loaded() => "loaded",
                Some(_) => "loading",
            },
        );

        !self.overmap_assets.is_empty()
            && !self.overmap_buffer_assets.is_empty()
            && !self.map_assets.is_empty()
            && !self.map_memory_assets.is_empty()
            && self.explored.as_deref().is_some_and(Explored::loaded)
            && self
                .subzone_level_entities
                .as_deref()
                .is_some_and(SubzoneLevelEntities::loaded)
    }
}
