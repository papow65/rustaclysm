use crate::application::ApplicationState;
use crate::gameplay::{
    Infos, LastSeen, Level, MissingAsset, ObjectCategory, ObjectName, SeenFrom, ZoneLevel,
    ZoneLevelIds, spawn::TileSpawner,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Res, StateScoped, Transform, Vec3, Visibility};
use bevy::render::view::RenderLayers;
use cdda_json_files::{CddaItemName, InfoId, ItemName, OvermapTerrainInfo};
use hud::HARD_TEXT_COLOR;

#[derive(SystemParam)]
pub(crate) struct ZoneSpawner<'w, 's> {
    infos: Res<'w, Infos>,
    pub(crate) zone_level_ids: Res<'w, ZoneLevelIds>,
    pub(crate) tile_spawner: TileSpawner<'w, 's>,
}

impl ZoneSpawner<'_, '_> {
    pub(crate) fn spawn_zone_level(
        &mut self,
        zone_level: ZoneLevel,
        child_visibiltiy: &Visibility,
    ) {
        //trace!("zone_level: {zone_level:?} {:?}", &definition);
        assert!(
            zone_level.level <= Level::ZERO,
            "Zone levels above ground may not be spawned"
        );

        let mut entity = self.tile_spawner.commands.spawn(zone_level);

        let Some(seen_from) = self
            .tile_spawner
            .explored
            .has_zone_level_been_seen(zone_level)
        else {
            entity.insert(MissingAsset);
            return;
        };

        let Some(info_id) = self.zone_level_ids.get(zone_level).cloned() else {
            entity.insert(MissingAsset);
            return;
        };

        let entity = entity.id();
        self.complete_zone_level(entity, zone_level, seen_from, &info_id, child_visibiltiy);
    }

    pub(crate) fn complete_zone_level(
        &mut self,
        entity: Entity,
        zone_level: ZoneLevel,
        seen_from: SeenFrom,
        info_id: &InfoId<OvermapTerrainInfo>,
        child_visibiltiy: &Visibility,
    ) {
        let name = ObjectName::new(
            self.infos.zone_levels.get(info_id).ok().map_or_else(
                || ItemName::from(CddaItemName::Simple(info_id.fallback_name())),
                |z| z.name.clone(),
            ),
            HARD_TEXT_COLOR,
        );

        let (seen_from, visibility) = match seen_from {
            SeenFrom::CloseBy | SeenFrom::FarAway => (LastSeen::Previously, Visibility::Inherited),
            SeenFrom::Never => (LastSeen::Never, Visibility::Hidden),
        };

        let pbr_bundles = self
            .tile_spawner
            .model_factory()
            .get_layers(info_id.untyped(), ObjectCategory::ZoneLevel, None)
            .map(|(mesh, transform, appearance)| {
                (
                    mesh,
                    transform,
                    appearance.fixed_material(),
                    *child_visibiltiy,
                    RenderLayers::layer(2),
                )
            });

        self.tile_spawner
            .commands
            .entity(entity)
            .insert((
                Transform {
                    translation: zone_level.base_corner().vec3() + Vec3::new(11.5, 0.0, 11.5),
                    scale: Vec3::splat(24.0),
                    ..Transform::default()
                },
                visibility,
                name,
                seen_from,
                StateScoped(ApplicationState::Gameplay),
            ))
            .with_children(|child_builder| {
                child_builder.spawn(pbr_bundles.base);
                if let Some(overlay_pbr_bundle) = pbr_bundles.overlay {
                    child_builder.spawn(overlay_pbr_bundle);
                }
            });
    }
}
