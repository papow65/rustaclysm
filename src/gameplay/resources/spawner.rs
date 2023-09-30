use crate::prelude::*;
use bevy::{
    ecs::system::{Insert, SystemParam},
    prelude::*,
    render::camera::{PerspectiveProjection, Projection::Perspective},
};

fn insert<T>(child_builder: &mut ChildBuilder, entity: Entity, bundle: T)
where
    T: Bundle + 'static,
{
    child_builder.add_command(Insert { entity, bundle });
}

#[derive(SystemParam)]
pub(crate) struct Spawner<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    location: ResMut<'w, Location>,
    pub(crate) explored: ResMut<'w, Explored>,
    sav: Res<'w, Sav>,
    model_factory: ModelFactory<'w>,
}

impl<'w, 's> Spawner<'w, 's> {
    fn spawn_character(
        &mut self,
        infos: &Infos,
        parent: Entity,
        pos: Pos,
        id: &ObjectId,
        name: Option<ObjectName>,
    ) -> Option<Entity> {
        let Some(character_info) = infos.character(id) else {
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
        let entity = self.spawn_tile(parent, pos, definition, object_name);

        let mut entity = self.commands.entity(entity);
        entity
            .insert(Life)
            .insert(Obstacle)
            .insert(Health(
                Limited::full(character_info.hp.unwrap_or(60) as u16),
            ))
            .insert(Stamina::Unlimited)
            .insert(WalkingMode::Running)
            .insert(faction.clone())
            .insert(Melee {
                dices: character_info.melee_dice,
                sides: character_info.melee_dice_sides,
            });

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
                .spawn(BodyContainers::default_hands_container())
                .insert(SpatialBundle::HIDDEN_IDENTITY)
                .set_parent(entity)
                .id();
            let clothing = self
                .commands
                .spawn(BodyContainers::default_clothing_container())
                .insert(SpatialBundle::HIDDEN_IDENTITY)
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
        let Some(field_info) = infos.field(id) else {
            println!("No info found for field {id:?}");
            return;
        };
        let object_name = ObjectName::new(field_info.name().clone(), BAD_TEXT_COLOR);

        let definition = &ObjectDefinition {
            category: ObjectCategory::Field,
            id: id.clone(),
        };
        self.spawn_tile(parent, pos, definition, object_name);
    }

    fn spawn_existing_item(
        &mut self,
        infos: &Infos,
        parent: Entity,
        pos: Pos,
        item: &CddaItem,
        amount: Amount,
    ) -> Result<(), LoadError> {
        let Some(item_info) = infos.item(&item.typeid) else {
            return Err(LoadError::new(format!(
                "No info found for {:?}. Spawning skipped",
                &item.typeid
            )));
        };

        //println!("{:?} {:?} {:?} {:?}", &parent, pos, &id, &amount);
        let definition = &ObjectDefinition {
            category: ObjectCategory::Item,
            id: item.typeid.clone(),
        };
        //println!("{:?} @ {pos:?}", &definition);
        let object_name = ObjectName::new(item_info.name.clone(), item_info.text_color());
        let entity = self.spawn_tile(parent, pos, definition, object_name);
        let mut entity = self.commands.entity(entity);
        entity.insert(amount);

        let (volume, mass) = match &item.corpse {
            Some(corpse_id) if corpse_id != &ObjectId::new("mon_null") => {
                println!("{:?}", &corpse_id);
                match infos.character(corpse_id) {
                    Some(monster_info) => (monster_info.volume, monster_info.mass),
                    None => (item_info.volume, item_info.mass),
                }
            }
            _ => (item_info.volume, item_info.mass),
        };
        entity.insert(Containable {
            // Based on cataclysm-ddasrc/mtype.cpp lines 47-48
            volume: volume.unwrap_or_else(|| Volume::from(String::from("62499 ml"))),
            mass: mass.unwrap_or_else(|| Mass::from(String::from("81499 g"))),
        });

        if let Some(item_tags) = &item.item_tags {
            if item_tags.contains(&String::from("FILTHY")) {
                entity.insert(Filthy);
            }
        }

        Ok(())
    }

    fn spawn_new_items(
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
                        self.spawn_existing_item(
                            infos,
                            parent_entity,
                            pos,
                            &CddaItem {
                                typeid: item.item.clone(),
                                snip_id: None,
                                charges: item.charges.as_ref().map(CountRange::random),
                                active: None,
                                corpse: None,
                                name: None,
                                owner: None,
                                bday: None,
                                last_temp_check: None,
                                specific_energy: None,
                                temperature: None,
                                item_vars: None,
                                item_tags: None,
                                contents: None,
                                components: None,
                                is_favorite: None,
                                relic_data: None,
                                damaged: None,
                                current_phase: None,
                                faults: None,
                                rot: None,
                                curammo: None,
                                item_counter: None,
                                variant: None,
                                recipe_charges: None,
                                poison: None,
                                burnt: None,
                                craft_data: None,
                                dropped_from: None,
                                degradation: None,
                            },
                            amount,
                        )
                        .expect("Existing item id");
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
        let item_group = &infos.item_group(id).expect("Existing item group");
        let items = item_group
            .items
            .as_ref()
            .or(item_group.entries.as_ref())
            .expect("items or entries");
        self.spawn_new_items(infos, parent_entity, pos, items);
    }

    fn spawn_furniture(&mut self, infos: &Infos, parent: Entity, pos: Pos, id: &ObjectId) {
        let Some(furniture_info) = infos.furniture(id) else {
            println!("No info found for {id:?}. Spawning skipped");
            return;
        };

        let definition = &ObjectDefinition {
            category: ObjectCategory::Furniture,
            id: id.clone(),
        };
        let object_name = ObjectName::new(furniture_info.name.clone(), DEFAULT_TEXT_COLOR);
        let tile = self.spawn_tile(parent, pos, definition, object_name);

        if !furniture_info.flags.transparent() {
            self.commands.entity(tile).insert(Opaque);
        }

        if furniture_info.bash.is_some() {
            // TODO
            self.commands
                .entity(tile)
                .insert(Integrity(Limited::full(10)));
        }

        match furniture_info.move_cost_mod {
            None => {}
            Some(MoveCostMod::Impossible(_)) => {
                self.commands.entity(tile).insert(Obstacle);
            }
            Some(MoveCostMod::Slower(increase)) => {
                self.commands.entity(tile).insert(Hurdle(increase));
            }
        }
    }

    pub(crate) fn spawn_terrain(&mut self, infos: &Infos, parent: Entity, pos: Pos, id: &ObjectId) {
        let Some(terrain_info) = infos.terrain(id) else {
            println!("No info found for terrain {:?}", &id);
            return;
        };

        let definition = &ObjectDefinition {
            category: ObjectCategory::Terrain,
            id: id.clone(),
        };
        let object_name = ObjectName::new(terrain_info.name.clone(), DEFAULT_TEXT_COLOR);
        let tile = self.spawn_tile(parent, pos, definition, object_name);

        if 0 < terrain_info.move_cost.0 {
            if terrain_info.close.is_some() {
                self.commands.entity(tile).insert(Closeable);
            }
            self.commands.entity(tile).insert(Accessible {
                water: terrain_info.flags.water(),
                move_cost: terrain_info.move_cost,
            });
        } else if terrain_info.open.is_some() {
            self.commands.entity(tile).insert(Openable);
        } else {
            self.commands.entity(tile).insert(Obstacle);
        }

        if !terrain_info.flags.transparent() {
            self.commands.entity(tile).insert(Opaque);
        }

        if terrain_info.flags.goes_up() {
            self.commands.entity(tile).insert(StairsUp);
        }

        if terrain_info.flags.goes_down() {
            self.commands.entity(tile).insert(StairsDown);
        } else {
            self.commands.entity(tile).insert(OpaqueFloor);
        }

        if let Some(bash) = &terrain_info.bash {
            if let Some(ter_set) = &bash.ter_set {
                if ter_set != &ObjectId::new("t_null") {
                    // TODO
                    self.commands
                        .entity(tile)
                        .insert(Integrity(Limited::full(10)));
                }
            }
        }
    }

    fn spawn_tile(
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

        let layers =
            self.model_factory
                .get_layers(definition, true)
                .map(|(pbr_bundle, apprearance)| {
                    (
                        pbr_bundle,
                        apprearance.clone(),
                        if last_seen == LastSeen::Never {
                            Some(apprearance.material(&LastSeen::Currently))
                            //None // TODO
                        } else {
                            Some(apprearance.material(&last_seen))
                        },
                    )
                });

        let tile = self
            .commands
            .spawn(SpatialBundle::default())
            .insert(Visibility::Hidden)
            .insert(definition.clone())
            .insert(pos)
            .insert(Transform::from_translation(pos.vec3()))
            .insert(object_name)
            .with_children(|child_builder| {
                {
                    let (pbr_bundle, appearance, material) = layers.base;
                    let child = child_builder.spawn(pbr_bundle).insert(appearance).id();
                    if let Some(material) = material {
                        insert(child_builder, child, material);
                    }
                }
                if let Some((pbr_bundle, appearance, material)) = layers.overlay {
                    let child = child_builder.spawn(pbr_bundle).insert(appearance).id();
                    if let Some(material) = material {
                        insert(child_builder, child, material);
                    }
                }
            })
            .id();
        //dbg!(tile);

        self.commands.entity(parent).add_child(tile);

        if definition.category.shading_applied() {
            self.commands.entity(tile).insert(last_seen);
        }

        self.location.update(tile, Some(pos));

        tile
    }

    fn configure_player(&mut self, player_entity: Entity) {
        let cursor_definition = ObjectDefinition {
            category: ObjectCategory::Meta,
            id: ObjectId::new("cursor"),
        };
        let mut cursor_bundle = self
            .model_factory
            .get_single_pbr_bundle(&cursor_definition, false);
        cursor_bundle.transform.translation.y = 0.1;
        cursor_bundle.transform.scale = Vec3::new(1.1, 1.0, 1.1);

        self.commands
            .entity(player_entity)
            .with_children(|child_builder| {
                child_builder
                    .spawn(SpatialBundle::default())
                    .insert(CameraBase)
                    .with_children(|child_builder| {
                        child_builder.spawn(cursor_bundle).insert(ExamineCursor);

                        let camera_direction = Transform::IDENTITY
                            .looking_at(Vec3::new(0.1, 0.0, -1.0), Vec3::Y)
                            * Transform::from_translation(Vec3::new(0.0, 0.3, 0.0));
                        child_builder
                            .spawn(PbrBundle {
                                transform: camera_direction,
                                ..PbrBundle::default()
                            })
                            .with_children(|child_builder| {
                                child_builder.spawn(Camera3dBundle {
                                    projection: Perspective(PerspectiveProjection {
                                        // more overview, less personal than the default
                                        fov: 0.3,
                                        ..default()
                                    }),
                                    ..default()
                                });
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
        self.commands
            .spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 50_000.0,
                    shadows_enabled: false, // TODO shadow direction does not match buildin shadows
                    ..DirectionalLight::default()
                },
                transform: light_transform,
                ..DirectionalLightBundle::default()
            })
            .insert(StateBound::<ApplicationState>::default());
    }

    pub(crate) fn spawn_characters(&mut self, infos: &Infos, offset: PosOffset) {
        let root = self
            .commands
            .spawn(SpatialBundle::default())
            .insert(StateBound::<ApplicationState>::default())
            .id();

        let player = self
            .spawn_character(
                infos,
                root,
                Pos::new(45, Level::ZERO, 56)
                    .offset(offset)
                    .unwrap()
                    .offset(PosOffset {
                        x: -9,
                        level: LevelOffset::ZERO,
                        z: 0,
                    })
                    .unwrap(),
                &ObjectId::new("human"),
                Some(ObjectName::from_str(
                    self.sav.player.name.as_str(),
                    GOOD_TEXT_COLOR,
                )),
            )
            .unwrap();
        self.commands
            .entity(player)
            .insert(Player)
            .insert(Stamina::Limited(Limited::full(300))) // override
            .insert(WalkingMode::Walking); // override
        self.configure_player(player);

        self.spawn_character(
            infos,
            root,
            Pos::new(10, Level::ZERO, 10).offset(offset).unwrap(),
            &ObjectId::new("human"),
            Some(ObjectName::from_str("Survivor", DEFAULT_TEXT_COLOR)),
        );

        self.spawn_character(
            infos,
            root,
            Pos::new(12, Level::ZERO, 16).offset(offset).unwrap(),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            Pos::new(40, Level::ZERO, 40).offset(offset).unwrap(),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            Pos::new(38, Level::ZERO, 39).offset(offset).unwrap(),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            Pos::new(37, Level::ZERO, 37).offset(offset).unwrap(),
            &ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            infos,
            root,
            Pos::new(34, Level::ZERO, 34).offset(offset).unwrap(),
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
            .furniture(&definition.id)
            .map(|f| f.bash.as_ref())
            .or_else(|| infos.terrain(&definition.id).map(|f| f.bash.as_ref()))
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
                    self.spawn_new_items(infos, parent_entity, pos, item_vec);
                }
                BashItems::Collection(id) => {
                    self.spawn_item_collection(infos, parent_entity, pos, id);
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct ZoneSpawner<'w, 's> {
    pub(crate) asset_server: Res<'w, AssetServer>,
    pub(crate) overmap_buffer_assets: Res<'w, Assets<OvermapBuffer>>,
    pub(crate) overmap_assets: Res<'w, Assets<Overmap>>,
    infos: Res<'w, Infos>,
    pub(crate) overmap_buffer_manager: ResMut<'w, OvermapBufferManager>,
    pub(crate) overmap_manager: ResMut<'w, OvermapManager>,
    pub(crate) map_manager: ResMut<'w, MapManager>,
    pub(crate) zone_level_ids: ResMut<'w, ZoneLevelIds>,
    pub(crate) zone_level_entities: ResMut<'w, ZoneLevelEntities>,
    pub(crate) subzone_level_entities: ResMut<'w, SubzoneLevelEntities>,
    pub(crate) spawner: Spawner<'w, 's>,
}

impl<'w, 's> ZoneSpawner<'w, 's> {
    pub(crate) fn spawn_expanded_subzone_level(
        &mut self,
        map_assets: &Assets<Map>,
        subzone_level: SubzoneLevel,
    ) {
        match self
            .map_manager
            .get_subzone_level(&self.asset_server, map_assets, subzone_level)
        {
            AssetState::Available { asset: submap } => {
                self.spawn_subzone(submap, subzone_level);
            }
            AssetState::Loading => {
                // It will be spawned when it's fully loaded.
            }
            AssetState::Nonexistent => {
                if let Some(object_id) = self.zone_level_ids.get(
                    &self.asset_server,
                    &self.overmap_assets,
                    &mut self.overmap_manager,
                    ZoneLevel::from(subzone_level),
                ) {
                    let fallback = Submap::fallback(subzone_level, object_id);
                    self.spawn_subzone(&fallback, subzone_level);
                }
            }
        }
    }

    pub(crate) fn spawn_subzone(&mut self, submap: &Submap, subzone_level: SubzoneLevel) {
        //println!("{:?} started...", subzone_level);
        let subzone_level_entity = self
            .spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(subzone_level)
            .insert(StateBound::<ApplicationState>::default())
            .id();

        if submap.terrain.is_significant() {
            let terrain = submap.terrain.load_as_subzone(subzone_level);
            let base_pos = subzone_level.base_pos();

            for x in 0..12 {
                for z in 0..12 {
                    let pos = base_pos
                        .offset(PosOffset {
                            x,
                            level: LevelOffset::ZERO,
                            z,
                        })
                        .unwrap();
                    //dbg!("{pos:?}");
                    let id = *terrain.get(&pos).unwrap();
                    if id != &ObjectId::new("t_open_air")
                        && id != &ObjectId::new("t_open_air_rooved")
                    {
                        self.spawner
                            .spawn_terrain(&self.infos, subzone_level_entity, pos, id);
                    }

                    for id in submap
                        .furniture
                        .iter()
                        .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                    {
                        self.spawner
                            .spawn_furniture(&self.infos, subzone_level_entity, pos, id);
                    }

                    for repetitions in submap
                        .items
                        .0
                        .iter()
                        .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                    {
                        for repetition in repetitions {
                            let CddaAmount { obj: item, amount } = repetition.as_amount();
                            //dbg!(&item.typeid);
                            if let Err(load_error) = self.spawner.spawn_existing_item(
                                &self.infos,
                                subzone_level_entity,
                                pos,
                                item,
                                Amount(item.charges.unwrap_or(1) * amount),
                            ) {
                                eprintln!("{load_error}");
                            }
                        }
                    }

                    for spawn in submap
                        .spawns
                        .iter()
                        .filter(|spawn| spawn.x == x && spawn.z == z)
                    {
                        //dbg!(&spawn.id);
                        self.spawner.spawn_character(
                            &self.infos,
                            subzone_level_entity,
                            pos,
                            &spawn.id,
                            None,
                        );
                    }

                    for fields in submap
                        .fields
                        .0
                        .iter()
                        .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                    {
                        //dbg!(&fields);
                        for field in &fields.0 {
                            self.spawner.spawn_field(
                                &self.infos,
                                subzone_level_entity,
                                pos,
                                &field.id,
                            );
                        }
                    }
                }
            }
            //println!("{:?} done", subzone_level);
        }

        self.subzone_level_entities
            .add(subzone_level, subzone_level_entity);
    }

    pub(crate) fn spawn_collapsed_zone_level(
        &mut self,
        zone_level: ZoneLevel,
        child_visibiltiy: &Visibility,
    ) {
        //println!("zone_level: {zone_level:?} {:?}", &definition);
        assert!(
            zone_level.level <= Level::ZERO,
            "Collapsed zone levels above ground may not be spawned"
        );

        let mut entity = self.spawner.commands.spawn(zone_level);
        self.zone_level_entities.add(zone_level, entity.id());

        let Some(seen_from) = self.spawner.explored.has_zone_level_been_seen(
            &self.asset_server,
            &self.overmap_buffer_assets,
            &mut self.overmap_buffer_manager,
            zone_level,
        ) else {
            entity.insert(MissingAsset);
            return;
        };

        let Some(definition) = self
            .zone_level_ids
            .get(
                &self.asset_server,
                &self.overmap_assets,
                &mut self.overmap_manager,
                zone_level,
            )
            .map(|object_id| ObjectDefinition {
                category: ObjectCategory::ZoneLevel,
                id: object_id.clone(),
            })
        else {
            entity.insert(MissingAsset);
            return;
        };

        let entity = entity.id();
        self.complete_collapsed_zone_level(
            entity,
            zone_level,
            seen_from,
            &definition,
            child_visibiltiy,
        );
    }

    pub(crate) fn complete_collapsed_zone_level(
        &mut self,
        entity: Entity,
        zone_level: ZoneLevel,
        seen_from: SeenFrom,
        definition: &ObjectDefinition,
        child_visibiltiy: &Visibility,
    ) {
        let zone_level_info = self.infos.zone_level(&definition.id);

        let name = ObjectName::new(
            zone_level_info.map_or_else(
                || ItemName::from(CddaItemName::Simple(definition.id.fallback_name())),
                |z| z.name.clone(),
            ),
            DEFAULT_TEXT_COLOR,
        );

        let mut entity = self.spawner.commands.entity(entity);
        entity.insert(Collapsed).insert(name);

        let pbr_bundles = self
            .spawner
            .model_factory
            .get_layers(definition, false)
            .map(|(pbr, _)| pbr);

        entity
            .insert(SpatialBundle::default())
            .insert(Transform {
                translation: zone_level.base_pos().vec3() + Vec3::new(11.5, 0.0, 11.5),
                scale: Vec3::splat(24.0),
                ..Transform::default()
            })
            .insert(match seen_from {
                SeenFrom::CloseBy | SeenFrom::FarAway => {
                    (LastSeen::Previously, Visibility::Inherited)
                }
                SeenFrom::Never => (LastSeen::Never, Visibility::Hidden),
            })
            .insert(StateBound::<ApplicationState>::default())
            .with_children(|child_builder| {
                child_builder
                    .spawn(pbr_bundles.base)
                    .insert(*child_visibiltiy);
                if let Some(pbr_bundle) = pbr_bundles.overlay {
                    child_builder.spawn(pbr_bundle).insert(*child_visibiltiy);
                }
            });
    }
}
