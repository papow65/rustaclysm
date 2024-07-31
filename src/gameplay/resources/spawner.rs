use crate::prelude::*;
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    render::{
        camera::{PerspectiveProjection, Projection::Perspective},
        view::RenderLayers,
    },
};

#[derive(SystemParam)]
pub(crate) struct Spawner<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    location: ResMut<'w, Location>,
    pub(crate) explored: ResMut<'w, Explored>,
    sav: Res<'w, Sav>,
    model_factory: ModelFactory<'w>,
}

impl<'w, 's> Spawner<'w, 's> {
    fn spawn_tile<'a>(
        &mut self,
        infos: &Infos,
        subzone_level_entity: Entity,
        pos: Pos,
        terrain_id: &ObjectId,
        furniture_ids: impl Iterator<Item = &'a ObjectId>,
        item_repetitions: impl Iterator<Item = &'a Vec<Repetition<CddaItem>>>,
        spawns: impl Iterator<Item = &'a Spawn>,
        fields: impl Iterator<Item = &'a FlatVec<Field, 3>>,
    ) {
        self.spawn_terrain(infos, subzone_level_entity, pos, terrain_id);

        for id in furniture_ids {
            self.spawn_furniture(infos, subzone_level_entity, pos, id);
        }

        for repetitions in item_repetitions {
            for repetition in repetitions {
                let CddaAmount { obj: item, amount } = repetition.as_amount();
                //dbg!(&item.typeid);
                _ = self.spawn_item(
                    infos,
                    subzone_level_entity,
                    pos,
                    item,
                    Amount(item.charges.unwrap_or(1) * amount),
                );
            }
        }

        for spawn in spawns {
            //dbg!(&spawn.id);
            self.spawn_character(infos, subzone_level_entity, pos, &spawn.id, None);
        }

        for fields in fields {
            //dbg!(&fields);
            for field in &fields.0 {
                self.spawn_field(infos, subzone_level_entity, pos, &field.id);
            }
        }
    }

    fn spawn_character(
        &mut self,
        infos: &Infos,
        parent: Entity,
        pos: Pos,
        id: &ObjectId,
        name: Option<ObjectName>,
    ) -> Option<Entity> {
        let Some(character_info) = infos.try_character(id) else {
            println!("No info found for {id:?}. Spawning skipped");
            return None;
        };
        let faction = match character_info.default_faction.as_str() {
            "human" => Faction::Human,
            "zombie" => Faction::Zombie,
            _ => Faction::Animal,
        };
        let object_name = ObjectName::new(character_info.name.clone(), faction.color());

        let definition = &ObjectDefinition {
            category: ObjectCategory::Character,
            id: id.clone(),
        };
        let entity = self.spawn_object(parent, pos, definition, object_name);
        let mut entity = self.commands.entity(entity);
        entity.insert((
            Life,
            Obstacle,
            Health(Limited::full(character_info.hp.unwrap_or(60) as u16)),
            Stamina::Unlimited,
            WalkingMode::Running,
            faction.clone(),
            Melee {
                dices: character_info.melee_dice,
                sides: character_info.melee_dice_sides,
            },
            HealingDuration::new(),
        ));

        if let Some(name) = name {
            entity.insert(name);
        }

        if 0 < character_info.speed {
            entity.insert(BaseSpeed::from_percent(character_info.speed));
        }

        if character_info.flags.aquatic() {
            entity.insert(Aquatic);
        }

        let entity = entity.id();

        if faction == Faction::Human {
            let hands = self
                .commands
                .spawn((
                    BodyContainers::default_hands_container_limits(),
                    SpatialBundle::HIDDEN_IDENTITY,
                ))
                .set_parent(entity)
                .id();
            let clothing = self
                .commands
                .spawn((
                    BodyContainers::default_clothing_container_limits(),
                    SpatialBundle::HIDDEN_IDENTITY,
                ))
                .set_parent(entity)
                .id();
            self.commands
                .entity(entity)
                .insert(BodyContainers { hands, clothing });
        }

        println!("Spawned a {:?} at {pos:?}", definition.id);
        Some(entity)
    }

    fn spawn_field(&mut self, infos: &Infos, parent: Entity, pos: Pos, id: &ObjectId) {
        let Some(field_info) = infos.try_field(id) else {
            println!("No info found for field {id:?}");
            return;
        };
        let object_name = ObjectName::new(field_info.name().clone(), BAD_TEXT_COLOR);

        let definition = &ObjectDefinition {
            category: ObjectCategory::Field,
            id: id.clone(),
        };
        self.spawn_object(parent, pos, definition, object_name);
    }

    pub(crate) fn spawn_item(
        &mut self,
        infos: &Infos,
        parent: Entity,
        pos: Pos,
        item: &CddaItem,
        amount: Amount,
    ) -> Result<Entity, ()> {
        let Some(item_info) = infos.try_item(&item.typeid) else {
            eprintln!("No info found for {:?}. Spawning skipped", &item.typeid);
            return Err(());
        };

        //println!("{:?} {:?} {:?} {:?}", &parent, pos, &id, &amount);
        let definition = &ObjectDefinition {
            category: ObjectCategory::Item,
            id: item.typeid.clone(),
        };
        //println!("{:?} @ {pos:?}", &definition);
        let object_name = ObjectName::new(item_info.name.clone(), item_info.text_color());
        let object_entity = self.spawn_object(parent, pos, definition, object_name);

        let (volume, mass) = match &item.corpse {
            Some(corpse_id) if corpse_id != &ObjectId::new("mon_null") => {
                println!("{:?}", &corpse_id);
                match infos.try_character(corpse_id) {
                    Some(monster_info) => (monster_info.volume, monster_info.mass),
                    None => (item_info.volume, item_info.mass),
                }
            }
            _ => (item_info.volume, item_info.mass),
        };

        let mut entity = self.commands.entity(object_entity);
        entity.insert((
            amount,
            Containable {
                // Based on cataclysm-ddasrc/mtype.cpp lines 47-48
                volume: volume.unwrap_or_else(|| Volume::from("62499 ml")),
                mass: mass.unwrap_or_else(|| Mass::from("81499 g")),
            },
        ));

        if item.item_tags.contains(&String::from("FILTHY")) {
            entity.insert(Filthy);
        }

        Ok(entity.id())
    }

    fn spawn_bashing_items(
        &mut self,
        infos: &Infos,
        parent_entity: Entity,
        pos: Pos,
        items: &Vec<BashItem>,
    ) {
        for item in items {
            match item {
                BashItem::Single(ref item) => {
                    if match &item.prob {
                        Some(probability) => probability.random(),
                        None => true,
                    } {
                        let amount = Amount(match &item.count {
                            Some(count) => count.random(),
                            None => 1,
                        });
                        let mut cdda_item = CddaItem::from(item.item.clone());
                        cdda_item.charges = item.charges.as_ref().map(CountRange::random);
                        _ = self.spawn_item(infos, parent_entity, pos, &cdda_item, amount);
                    }
                }
                BashItem::Group { ref group } => {
                    self.spawn_item_collection(infos, parent_entity, pos, group);
                }
            }
        }
    }

    fn spawn_item_collection(
        &mut self,
        infos: &Infos,
        parent_entity: Entity,
        pos: Pos,
        id: &ObjectId,
    ) {
        let item_group = &infos.item_group(id);
        let items = item_group
            .items
            .as_ref()
            .or(item_group.entries.as_ref())
            .expect("items or entries");
        self.spawn_bashing_items(infos, parent_entity, pos, items);
    }

    fn spawn_furniture(&mut self, infos: &Infos, parent: Entity, pos: Pos, id: &ObjectId) {
        let Some(furniture_info) = infos.try_furniture(id) else {
            println!("No info found for {id:?}. Spawning skipped");
            return;
        };

        let definition = &ObjectDefinition {
            category: ObjectCategory::Furniture,
            id: id.clone(),
        };
        let object_name = ObjectName::new(furniture_info.name.clone(), DEFAULT_TEXT_COLOR);
        let object_entity = self.spawn_object(parent, pos, definition, object_name);

        if !furniture_info.flags.transparent() {
            self.commands.entity(object_entity).insert(Opaque);
        }

        if furniture_info.bash.is_some() {
            // TODO
            self.commands
                .entity(object_entity)
                .insert(Integrity(Limited::full(10)));
        }

        match furniture_info.move_cost_mod {
            None => {}
            Some(MoveCostMod::Impossible(_)) => {
                self.commands.entity(object_entity).insert(Obstacle);
            }
            Some(MoveCostMod::Slower(increase)) => {
                self.commands.entity(object_entity).insert(Hurdle(increase));
            }
        }
    }

    pub(crate) fn spawn_terrain(&mut self, infos: &Infos, parent: Entity, pos: Pos, id: &ObjectId) {
        let Some(terrain_info) = infos.try_terrain(id) else {
            println!("No info found for terrain {:?}", &id);
            return;
        };

        if id == &ObjectId::new("t_open_air") || id == &ObjectId::new("t_open_air_rooved") {
            // Don't spawn air terrain to keep the entity count low
            return;
        }

        let definition = &ObjectDefinition {
            category: ObjectCategory::Terrain,
            id: id.clone(),
        };
        let object_name = ObjectName::new(terrain_info.name.clone(), DEFAULT_TEXT_COLOR);
        let object_entity = self.spawn_object(parent, pos, definition, object_name);

        if terrain_info.move_cost.accessible() {
            if terrain_info.close.is_some() {
                self.commands.entity(object_entity).insert(Closeable);
            }
            self.commands.entity(object_entity).insert(Accessible {
                water: terrain_info.flags.water(),
                move_cost: terrain_info.move_cost,
            });
        } else if terrain_info.open.is_some() {
            self.commands.entity(object_entity).insert(Openable);
        } else {
            self.commands.entity(object_entity).insert(Obstacle);
        }

        if !terrain_info.flags.transparent() {
            self.commands.entity(object_entity).insert(Opaque);
        }

        if terrain_info.flags.goes_up() {
            self.commands.entity(object_entity).insert(StairsUp);
        }

        if terrain_info.flags.goes_down() {
            self.commands.entity(object_entity).insert(StairsDown);
        } else {
            self.commands.entity(object_entity).insert(OpaqueFloor);
        }

        if let Some(bash) = &terrain_info.bash {
            if let Some(ter_set) = &bash.ter_set {
                if ter_set != &ObjectId::new("t_null") {
                    // TODO
                    self.commands
                        .entity(object_entity)
                        .insert(Integrity(Limited::full(10)));
                }
            }
        }
    }

    fn spawn_object(
        &mut self,
        parent: Entity,
        pos: Pos,
        definition: &ObjectDefinition,
        object_name: ObjectName,
    ) -> Entity {
        //dbg!(&parent);
        //dbg!(pos);
        //dbg!(&definition);
        let last_seen = if definition.category.shading_applied() {
            if self.explored.has_pos_been_seen(pos) {
                LastSeen::Previously
            } else {
                LastSeen::Never
            }
        } else {
            // cursor -> dummy value that gives normal material
            LastSeen::Currently
        };
        //dbg!(&last_seen);

        let layers = self
            .model_factory
            .get_layers(definition, Visibility::Inherited, true)
            .map(|(mut pbr_bundle, apprearance)| {
                pbr_bundle.material = if last_seen == LastSeen::Never {
                    apprearance.material(&LastSeen::Currently)
                } else {
                    apprearance.material(&last_seen)
                };
                (pbr_bundle, apprearance.clone(), RenderLayers::layer(1))
            });

        let mut entity_commands = self.commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(pos.vec3()),
                ..SpatialBundle::HIDDEN_IDENTITY
            },
            definition.clone(),
            pos,
            object_name,
        ));
        entity_commands.with_children(|child_builder| {
            child_builder.spawn(layers.base);
            if let Some(overlay) = layers.overlay {
                child_builder.spawn(overlay);
            }
        });
        if definition.category.shading_applied() {
            entity_commands.insert(last_seen);
        }

        let entity = entity_commands.id();
        self.commands.entity(parent).add_child(entity);
        self.location.update(entity, Some(pos));

        entity
    }

    fn configure_player(&mut self, player_entity: Entity) {
        let cursor_definition = ObjectDefinition {
            category: ObjectCategory::Meta,
            id: ObjectId::new("cursor"),
        };
        let mut cursor_bundle = self.model_factory.get_single_pbr_bundle(&cursor_definition);
        cursor_bundle.transform.translation.y = 0.1;
        cursor_bundle.transform.scale = Vec3::new(1.1, 1.0, 1.1);

        self.commands
            .entity(player_entity)
            .with_children(|child_builder| {
                child_builder
                    .spawn(SpatialBundle::default())
                    .insert(CameraBase)
                    .with_children(|child_builder| {
                        child_builder.spawn((cursor_bundle, ExamineCursor));

                        let camera_direction = Transform::IDENTITY
                            .looking_at(Vec3::new(0.1, 0.0, -1.0), Vec3::Y)
                            * Transform::from_translation(Vec3::new(0.0, 0.3, 0.0));
                        child_builder
                            .spawn(PbrBundle {
                                transform: camera_direction,
                                ..PbrBundle::default()
                            })
                            .with_children(|child_builder| {
                                println!("Camera");
                                child_builder.spawn((
                                    Camera3dBundle {
                                        projection: Perspective(PerspectiveProjection {
                                            // more overview, less personal than the default
                                            fov: 0.3,
                                            ..default()
                                        }),
                                        ..default()
                                    },
                                    RenderLayers::default().with(1).without(2),
                                ));
                            });
                    });
            });
    }

    pub(crate) fn spawn_light(&mut self) {
        let light_transform = Transform::from_matrix(Mat4::from_euler(
            EulerRot::ZYX,
            0.0,
            -0.18 * std::f32::consts::TAU,
            -std::f32::consts::FRAC_PI_4,
        ));
        //dbg!(&light_transform);
        self.commands.spawn((
            DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 10_000.0,
                    shadows_enabled: false, // TODO shadow direction does not match buildin shadows
                    ..DirectionalLight::default()
                },
                transform: light_transform,
                ..DirectionalLightBundle::default()
            },
            StateScoped(ApplicationState::Gameplay),
        ));
    }

    pub(crate) fn spawn_characters(&mut self, infos: &Infos, spawn_pos: Pos) {
        let root = self
            .commands
            .spawn(SpatialBundle::default())
            .insert(StateScoped(ApplicationState::Gameplay))
            .id();

        let player = self
            .spawn_character(
                infos,
                root,
                spawn_pos.horizontal_offset(36, 56),
                &ObjectId::new("human"),
                Some(ObjectName::from_str(
                    self.sav.player.name.as_str(),
                    GOOD_TEXT_COLOR,
                )),
            )
            .expect("Player character should be spawned");
        self.commands
            .entity(player)
            .insert(Player)
            .insert(Stamina::Limited(Limited::full(300))) // override
            .insert(WalkingMode::Walking); // override
        self.configure_player(player);

        self.spawn_character(
            infos,
            root,
            spawn_pos.horizontal_offset(10, 10),
            &ObjectId::new("human"),
            Some(ObjectName::from_str("Survivor", DEFAULT_TEXT_COLOR)),
        );

        self.spawn_character(
            infos,
            root,
            spawn_pos.horizontal_offset(12, 16),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            spawn_pos.horizontal_offset(40, 40),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            spawn_pos.horizontal_offset(38, 39),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            spawn_pos.horizontal_offset(37, 37),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            spawn_pos.horizontal_offset(34, 34),
            &ObjectId::new("mon_zombie"),
            None,
        );
    }

    pub(crate) fn spawn_smashed(
        &mut self,
        infos: &Infos,
        parent_entity: Entity,
        pos: Pos,
        definition: &ObjectDefinition,
    ) {
        assert!(
            matches!(
                definition.category,
                ObjectCategory::Furniture | ObjectCategory::Terrain
            ),
            "Only furniture and terrain can be smashed"
        );

        let bash = infos
            .try_furniture(&definition.id)
            .map(|f| f.bash.as_ref())
            .or_else(|| infos.try_terrain(&definition.id).map(|f| f.bash.as_ref()))
            .expect("Furniture, or terrain")
            .expect("Smashable");

        if let Some(terrain_id) = &bash.ter_set {
            assert_eq!(
                definition.category,
                ObjectCategory::Terrain,
                "The terrain field requires a terrain category"
            );
            self.spawn_terrain(infos, parent_entity, pos, terrain_id);
        } else if let Some(furniture_id) = &bash.furn_set {
            assert_eq!(
                definition.category,
                ObjectCategory::Furniture,
                "The furniture field requires a furniture category"
            );
            self.spawn_furniture(infos, parent_entity, pos, furniture_id);
        }

        if let Some(items) = &bash.items {
            match items {
                BashItems::Explicit(item_vec) => {
                    self.spawn_bashing_items(infos, parent_entity, pos, item_vec);
                }
                BashItems::Collection(id) => {
                    self.spawn_item_collection(infos, parent_entity, pos, id);
                }
            }
        }
    }

    pub(crate) fn spawn_craft(
        &mut self,
        infos: &Infos,
        parent_entity: Entity,
        pos: Pos,
        object_id: ObjectId,
    ) -> Entity {
        let craft = CddaItem::from(ObjectId::new("craft"));
        let entity = self
            .spawn_item(infos, parent_entity, pos, &craft, Amount::SINGLE)
            .expect("Spawning craft item should have succeeded");

        let recipe = infos.recipe(&object_id);
        let crafting_time = recipe.time.expect("Craftable recipes should have a time");
        let craft = Craft::new(object_id, crafting_time);
        self.commands.entity(entity).insert(craft);

        entity
    }
}

#[derive(SystemParam)]
pub(crate) struct SubzoneSpawner<'w, 's> {
    infos: Res<'w, Infos>,
    zone_level_ids: ResMut<'w, ZoneLevelIds>,
    pub(crate) subzone_level_entities: ResMut<'w, SubzoneLevelEntities>,
    overmap_manager: OvermapManager<'w>,
    spawner: Spawner<'w, 's>,
}

impl<'w, 's> SubzoneSpawner<'w, 's> {
    pub(crate) fn spawn_subzone_level(
        &mut self,
        map_manager: &mut MapManager,
        map_memory_manager: &mut MapMemoryManager,
        subzone_level: SubzoneLevel,
    ) {
        if self.subzone_level_entities.get(subzone_level).is_some() {
            eprintln!("{subzone_level:?} already exists");
            return;
        }

        match map_manager.submap(subzone_level) {
            AssetState::Available { asset: submap } => {
                match map_memory_manager.submap(subzone_level) {
                    AssetState::Available { .. } | AssetState::Nonexistent => {
                        self.spawn_submap(submap, subzone_level);
                    }
                    AssetState::Loading => {
                        // It will be spawned when it's fully loaded.
                    }
                }
            }
            AssetState::Loading => {
                // It will be spawned when it's fully loaded.
            }
            AssetState::Nonexistent => {
                if let Some(object_id) = self
                    .zone_level_ids
                    .get(&mut self.overmap_manager, ZoneLevel::from(subzone_level))
                {
                    let submap = Submap::fallback(subzone_level, object_id);
                    self.spawn_submap(&submap, subzone_level);
                }
            }
        }
    }

    fn spawn_submap(&mut self, submap: &Submap, subzone_level: SubzoneLevel) {
        //println!("{:?} started...", subzone_level);
        let subzone_level_entity = self
            .spawner
            .commands
            .spawn((
                SpatialBundle::default(),
                subzone_level,
                StateScoped(ApplicationState::Gameplay),
            ))
            .id();

        if submap.terrain.is_significant() {
            let terrain = submap.terrain.load_as_subzone(subzone_level);
            let base_pos = subzone_level.base_corner();

            for x in 0..12 {
                for z in 0..12 {
                    let pos_offset = PosOffset {
                        x,
                        level: LevelOffset::ZERO,
                        z,
                    };
                    let pos = base_pos.horizontal_offset(x, z);
                    //dbg!("{pos:?}");
                    let terrain_id = *terrain.get(&pos).expect("Terrain id should be found");
                    let furniture_ids = submap.furniture.iter().filter_map(|at| at.get(pos_offset));
                    let item_repetitions =
                        submap.items.0.iter().filter_map(|at| at.get(pos_offset));
                    let spawns = submap
                        .spawns
                        .iter()
                        .filter(|spawn| spawn.x == pos_offset.x && spawn.z == pos_offset.z);
                    let fields = submap.fields.0.iter().filter_map(|at| at.get(pos_offset));
                    self.spawner.spawn_tile(
                        &self.infos,
                        subzone_level_entity,
                        pos,
                        terrain_id,
                        furniture_ids,
                        item_repetitions,
                        spawns,
                        fields,
                    );
                }
            }

            if let AssetState::Available { asset: overmap } = self
                .overmap_manager
                .get(Overzone::from(ZoneLevel::from(subzone_level).zone))
            {
                let spawned_offset = SubzoneOffset::from(subzone_level);
                for monster in overmap
                    .monster_map
                    .0
                    .iter()
                    .filter(|(at, _)| at == &spawned_offset)
                    .map(|(_, monster)| monster)
                {
                    eprintln!("TODO spanw {monster:?} at {spawned_offset:?}");
                    self.spawner.spawn_character(
                        &self.infos,
                        subzone_level_entity,
                        base_pos,
                        &monster.typeid,
                        None,
                    );
                }
            }

            //println!("{:?} done", subzone_level);
        }

        self.subzone_level_entities
            .add(subzone_level, subzone_level_entity);
    }
}

#[derive(SystemParam)]
pub(crate) struct ZoneSpawner<'w, 's> {
    infos: Res<'w, Infos>,
    pub(crate) zone_level_ids: ResMut<'w, ZoneLevelIds>,
    pub(crate) overmap_manager: OvermapManager<'w>,
    pub(crate) overmap_buffer_manager: OvermapBufferManager<'w>,
    pub(crate) spawner: Spawner<'w, 's>,
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

        let mut entity = self.spawner.commands.spawn(zone_level);

        let Some(seen_from) = self
            .spawner
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
            DEFAULT_TEXT_COLOR,
        );

        let (seen_from, visibility) = match seen_from {
            SeenFrom::CloseBy | SeenFrom::FarAway => (LastSeen::Previously, Visibility::Inherited),
            SeenFrom::Never => (LastSeen::Never, Visibility::Hidden),
        };

        let pbr_bundles = self
            .spawner
            .model_factory
            .get_layers(definition, *child_visibiltiy, false)
            .map(|(pbr, _)| (pbr, RenderLayers::layer(2)));

        self.spawner
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
