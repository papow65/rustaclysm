use crate::application::ApplicationState;
use crate::gameplay::{
    Infos, LastSeen, Level, MissingAsset, ObjectCategory, ObjectDefinition, ObjectName,
    OvermapBufferManager, OvermapManager, SeenFrom, TileSpawner, ZoneLevel, ZoneLevelIds,
};
use crate::hud::HARD_TEXT_COLOR;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    BuildChildren, ChildBuild, Entity, Res, ResMut, SpatialBundle, StateScoped, Transform, Vec3,
    Visibility,
};
use bevy::render::view::RenderLayers;
use cdda_json_files::{CddaItemName, ItemName};

#[derive(SystemParam)]
pub(crate) struct ZoneSpawner<'w, 's> {
    infos: Res<'w, Infos>,
    pub(crate) zone_level_ids: ResMut<'w, ZoneLevelIds>,
    pub(crate) overmap_manager: OvermapManager<'w>,
    pub(crate) overmap_buffer_manager: OvermapBufferManager<'w>,
    pub(crate) tile_spawner: TileSpawner<'w, 's>,
}

impl<'w, 's> ZoneSpawner<'w, 's> {
    pub(crate) fn spawn_zone_level(
        &mut self,
        zone_level: ZoneLevel,
        child_visibiltiy: &Visibility,
    ) {
        //println!("zone_level: {zone_level:?} {:?}", &definition);
        assert!(
            zone_level.level <= Level::ZERO,
            "Zone levels above ground may not be spawned"
        );

        let mut entity = self.tile_spawner.commands.spawn(zone_level);

        let Some(seen_from) = self
            .tile_spawner
            .explored
            .has_zone_level_been_seen(&mut self.overmap_buffer_manager, zone_level)
        else {
            entity.insert(MissingAsset);
            return;
        };

        let Some(definition) = self
            .zone_level_ids
            .get(&mut self.overmap_manager, zone_level)
            .map(|object_id| ObjectDefinition {
                category: ObjectCategory::ZoneLevel,
                id: object_id.clone(),
            })
        else {
            entity.insert(MissingAsset);
            return;
        };

        let entity = entity.id();
        self.complete_zone_level(entity, zone_level, seen_from, &definition, child_visibiltiy);
    }

    pub(crate) fn complete_zone_level(
        &mut self,
        entity: Entity,
        zone_level: ZoneLevel,
        seen_from: SeenFrom,
        definition: &ObjectDefinition,
        child_visibiltiy: &Visibility,
    ) {
        let zone_level_info = self.infos.try_zone_level(&definition.id);

        let name = ObjectName::new(
            zone_level_info.map_or_else(
                || ItemName::from(CddaItemName::Simple(definition.id.fallback_name())),
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
            .get_layers(definition, *child_visibiltiy, false, None)
            .map(|(pbr, _)| (pbr, RenderLayers::layer(2)));

        self.tile_spawner
            .commands
            .entity(entity)
            .insert((
                SpatialBundle {
                    transform: Transform {
                        translation: zone_level.base_corner().vec3() + Vec3::new(11.5, 0.0, 11.5),
                        scale: Vec3::splat(24.0),
                        ..Transform::default()
                    },
                    visibility,
                    ..SpatialBundle::default()
                },
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
