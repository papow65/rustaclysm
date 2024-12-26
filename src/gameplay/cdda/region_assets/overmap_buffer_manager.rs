use crate::gameplay::cdda::paths::OvermapBufferPath;
use crate::gameplay::cdda::region_assets::AssetStorage;
use crate::gameplay::{
    ActiveSav, AssetState, Exploration, Level, OvermapBufferAsset, Overzone,
    RepetitionBlockExt as _,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{AssetEvent, AssetServer, Assets, EventReader, Res, ResMut};

#[derive(SystemParam)]
pub(crate) struct OvermapBufferManager<'w, 's> {
    asset_events: EventReader<'w, 's, AssetEvent<OvermapBufferAsset>>,
    active_sav: Res<'w, ActiveSav>,
    storage: ResMut<'w, AssetStorage<OvermapBufferAsset>>,
    asset_server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<OvermapBufferAsset>>,
}

impl OvermapBufferManager<'_, '_> {
    pub(crate) fn load(&mut self, overzone: Overzone) -> AssetState<OvermapBufferAsset> {
        let path = OvermapBufferPath::new(&self.active_sav.sav_path(), overzone);
        self.storage
            .handle(&self.asset_server, &self.assets, overzone, path)
    }

    pub(crate) fn read_seen_zone_levels(
        &mut self,
    ) -> impl Iterator<Item = Exploration> + use<'_> + '_ {
        self.asset_events.read().filter_map(|event| {
            if let AssetEvent::LoadedWithDependencies { id } = event {
                let overzone = self.storage.region(id);
                match (overzone, self.assets.get(*id)) {
                    (Some(overzone), Some(OvermapBufferAsset(overmap_buffer))) => {
                        Some(Exploration::Overzone(
                            overzone,
                            Level::GROUNDS
                                .iter()
                                .flat_map(move |level| {
                                    overmap_buffer
                                        .visible
                                        .get(level.index())
                                        .expect("All levels should be present")
                                        .load_as_overzone(overzone, *level)
                                        .into_iter()
                                        .filter_map(|(zone_level, seen)| seen.then_some(zone_level))
                                })
                                .collect(),
                        ))
                    }
                    (None, None) => None,
                    unexpected => panic!("{unexpected:?}"),
                }
            } else {
                None
            }
        })
    }
}
