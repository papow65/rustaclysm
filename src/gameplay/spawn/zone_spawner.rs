use crate::application::ApplicationState;
use crate::gameplay::{
    spawn::TileSpawner, Infos, LastSeen, Level, MissingAsset, ObjectCategory, ObjectDefinition,
    ObjectName, SeenFrom, ZoneLevel, ZoneLevelIds,
};
use crate::hud::HARD_TEXT_COLOR;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    BuildChildren as _, ChildBuild as _, Entity, Res, StateScoped, Transform, Vec3, Visibility,
};
use bevy::render::view::RenderLayers;
use cdda_json_files::{CddaItemName, ItemName};

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
        //println!("zone_level: {zone_level:?} {:?}", &definition);
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

        let Some(definition) =
            self.zone_level_ids
                .get(zone_level)
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
            .get_layers(definition, None)
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
