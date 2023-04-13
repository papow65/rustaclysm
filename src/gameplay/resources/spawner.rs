use crate::prelude::*;
use bevy::{
    ecs::system::{Insert, SystemParam},
    prelude::*,
    render::camera::{PerspectiveProjection, Projection::Perspective},
};
use std::cmp::Ordering;

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
    pub(crate) infos: Res<'w, Infos>,
    sav: Res<'w, Sav>,
    model_factory: ModelFactory<'w>,
}

impl<'w, 's> Spawner<'w, 's> {
    fn spawn_character(
        &mut self,
        parent: Entity,
        pos: Pos,
        id: ObjectId,
        name: Option<ObjectName>,
    ) -> Option<Entity> {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Character,
            id,
        };

        let Some(character_info) = self
            .infos
            .character(&definition.id) else {
                println!("No info found for {:?}. Spawning skipped", definition.id);
                return None;
            };
        let faction = match character_info.default_faction.as_str() {
            "human" => Faction::Human,
            "zombie" => Faction::Zombie,
            _ => Faction::Animal,
        };

        let entity = self.spawn_tile(parent, pos, definition, Some(faction.color()));

        // duplicated lookup to make the borrow checker happy
        let character_info = self.infos.character(&definition.id).unwrap();
        self.commands
            .entity(entity)
            .insert(Obstacle)
            .insert(Health(
                Limited::full(character_info.hp.unwrap_or(60) as u16),
            ))
            .insert(Stamina::Unlimited)
            .insert(WalkingMode::Running)
            .insert(faction)
            .insert(Hands::default())
            .insert(Clothing::default())
            .insert(Melee {
                dices: character_info.melee_dice,
                sides: character_info.melee_dice_sides,
            });

        if let Some(name) = name {
            self.commands.entity(entity).insert(name);
        }

        if 0 < character_info.speed {
            self.commands
                .entity(entity)
                .insert(BaseSpeed::from_percent(character_info.speed));
        }

        if character_info.flags.aquatic() {
            self.commands.entity(entity).insert(Aquatic);
        }

        println!("Spawned a {:?} at {pos:?}", definition.id);
        Some(entity)
    }

    fn spawn_field(&mut self, parent: Entity, pos: Pos, id: ObjectId) {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Field,
            id,
        };

        let Some(_field_info) = self
        .infos
        .field(&definition.id) else {
            println!("No info found for field {:?}", definition.id);
            return;
        };

        self.spawn_tile(parent, pos, definition, None);
    }

    fn spawn_existing_item(
        &mut self,
        parent: Entity,
        pos: Pos,
        item: &CddaItem,
        amount: Amount,
    ) -> Result<(), LoadError> {
        //println!("{:?} {:?} {:?} {:?}", &parent, pos, &id, &amount);
        let definition = &ObjectDefinition {
            category: ObjectCategory::Item,
            id: item.typeid.clone(),
        };

        //println!("{:?} @ {pos:?}", &definition);
        let entity = self.spawn_tile(parent, pos, definition, None);
        let mut entity = self.commands.entity(entity);
        entity.insert(amount);

        let Some(item_info) = self
            .infos
            .item(&definition.id) else {
                entity.despawn_recursive();
                return Err(LoadError::new(format!("No info found for {:?}. Spawning skipped", definition.id)));
        };
        let (volume, mass) = match &item.corpse {
            Some(corpse_id) if corpse_id != &ObjectId::new("mon_null") => {
                println!("{:?}", &corpse_id);
                match self.infos.character(corpse_id) {
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

    fn spawn_furniture(&mut self, parent: Entity, pos: Pos, id: ObjectId) {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Furniture,
            id,
        };

        let tile = self.spawn_tile(parent, pos, definition, None);

        let Some(furniture_info) = self
        .infos
        .furniture(&definition.id) else {
            self.commands.entity(tile).despawn_recursive();
            println!("No info found for {:?}. Spawning skipped", definition.id);
            return;
        };

        if !furniture_info.flags.transparent() {
            self.commands.entity(tile).insert(Opaque);
        }

        if furniture_info.bash.is_some() {
            // TODO
            self.commands
                .entity(tile)
                .insert(Integrity(Limited::full(10)));
        }

        match furniture_info.move_cost_mod.0.cmp(&0) {
            Ordering::Less => {
                self.commands.entity(tile).insert(Obstacle);
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                self.commands
                    .entity(tile)
                    .insert(Hurdle(furniture_info.move_cost_mod));
            }
        }
    }

    pub(crate) fn spawn_terrain(&mut self, parent: Entity, pos: Pos, id: ObjectId) {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Terrain,
            id,
        };

        let tile = self.spawn_tile(parent, pos, definition, None);

        let Some(terrain_info) = self
            .infos
            .terrain(&definition.id) else {
                self.commands.entity(tile).despawn_recursive();
                println!("No info found for terrain {:?}", definition.id);
                return;
        };

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
        color: Option<Color>,
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

        let tile = self
            .commands
            .spawn(SpatialBundle::default())
            .insert(Visibility::Hidden)
            .insert(definition.clone())
            .insert(pos)
            .insert(Transform::from_translation(pos.vec3()))
            .insert(self.infos.name(definition, color))
            .with_children(|child_builder| {
                for (pbr_bundle, apprearance) in
                    self.model_factory.get_model_bundles(definition, true)
                {
                    let material = if last_seen == LastSeen::Never {
                        None
                    } else {
                        Some(apprearance.material(&last_seen))
                    };
                    let child = child_builder.spawn(pbr_bundle).insert(apprearance).id();
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

    pub(crate) fn spawn_light(&mut self, parent: Entity) {
        let light_transform = Transform::from_matrix(Mat4::from_euler(
            EulerRot::ZYX,
            0.0,
            -0.18 * std::f32::consts::TAU,
            -std::f32::consts::FRAC_PI_4,
        ));
        //dbg!(&light_transform);
        let light = self
            .commands
            .spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    illuminance: 50_000.0,
                    shadows_enabled: false, // TODO shadow direction does not match buildin shadows
                    ..DirectionalLight::default()
                },
                transform: light_transform,
                ..DirectionalLightBundle::default()
            })
            .id();

        self.commands.entity(parent).add_child(light);
    }

    pub(crate) fn spawn_characters(&mut self, parent: Entity, offset: PosOffset) {
        let player = self
            .spawn_character(
                parent,
                Pos::new(45, Level::ZERO, 56)
                    .offset(offset)
                    .unwrap()
                    .offset(PosOffset {
                        x: -9,
                        level: LevelOffset::ZERO,
                        z: 0,
                    })
                    .unwrap(),
                ObjectId::new("human"),
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
            parent,
            Pos::new(10, Level::ZERO, 10).offset(offset).unwrap(),
            ObjectId::new("human"),
            Some(ObjectName::from_str("Survivor", DEFAULT_TEXT_COLOR)),
        );

        self.spawn_character(
            parent,
            Pos::new(12, Level::ZERO, 16).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            parent,
            Pos::new(40, Level::ZERO, 40).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            parent,
            Pos::new(38, Level::ZERO, 39).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            parent,
            Pos::new(37, Level::ZERO, 37).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
            None,
        );
        self.spawn_character(
            parent,
            Pos::new(34, Level::ZERO, 34).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
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
        assert!(matches!(
            definition.category,
            ObjectCategory::Furniture | ObjectCategory::Terrain
        ));

        let bash = infos
            .furniture(&definition.id)
            .map(|f| f.bash.as_ref())
            .or_else(|| infos.terrain(&definition.id).map(|f| f.bash.as_ref()))
            .expect("Furniture, or terrain")
            .expect("Smashable");

        if let Some(terrain_id) = &bash.ter_set {
            assert!(definition.category == ObjectCategory::Terrain);
            self.spawn_terrain(parent_entity, pos, terrain_id.clone());
        }

        if let Some(furniture_id) = &bash.furn_set {
            assert!(definition.category == ObjectCategory::Furniture);
            self.spawn_furniture(parent_entity, pos, furniture_id.clone());
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

#[derive(Default, Resource)]
pub(crate) struct Maps {
    pub(crate) loading: Vec<Handle<Map>>,
}

#[derive(SystemParam)]
pub(crate) struct ZoneSpawner<'w, 's> {
    pub(crate) asset_server: Res<'w, AssetServer>,
    paths: Res<'w, Paths>,
    pub(crate) maps: ResMut<'w, Maps>,
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
        let map_path = MapPath::new(&self.paths.world_path(), ZoneLevel::from(subzone_level));
        if map_path.0.exists() {
            let map_handle = self.asset_server.load(map_path.0.as_path());
            if let Some(map) = map_assets.get(&map_handle) {
                let submap = &map.0[subzone_level.index()];
                self.spawn_subzone(submap, subzone_level);
            } else {
                self.maps.loading.push(self.asset_server.load(map_path.0));
                // It will be spawned when it's fully loaded.
            }
        } else {
            let object_id = self.zone_level_ids.get(ZoneLevel::from(subzone_level));
            let fallback = Submap::fallback(subzone_level, object_id);
            self.spawn_subzone(&fallback, subzone_level);
        }
    }

    pub(crate) fn spawn_subzone(&mut self, submap: &Submap, subzone_level: SubzoneLevel) {
        //println!("{:?} started...", subzone_level);
        let subzone_level_entity = self
            .spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(subzone_level)
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
                            .spawn_terrain(subzone_level_entity, pos, id.clone());
                    }

                    for id in submap
                        .furniture
                        .iter()
                        .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                    {
                        self.spawner
                            .spawn_furniture(subzone_level_entity, pos, id.clone());
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
                            subzone_level_entity,
                            pos,
                            spawn.id.clone(),
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
                            self.spawner
                                .spawn_field(subzone_level_entity, pos, field.id.clone());
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
    ) -> Result<(), ()> {
        //println!("zone_level: {zone_level:?} {:?}", &definition);
        assert!(zone_level.level <= Level::ZERO);

        let Some(seen_from) = self
            .spawner.explored
            .has_zone_level_been_seen(&self.asset_server, zone_level) else {
                return Err(());
            };

        let definition = ObjectDefinition {
            category: ObjectCategory::ZoneLevel,
            id: self.zone_level_ids.get(zone_level).clone(),
        };

        let mut entity = self.spawner.commands.spawn(zone_level);
        entity
            .insert(Collapsed)
            .insert(self.spawner.infos.name(&definition, None));

        let pbr_bundles = self
            .spawner
            .model_factory
            .get_model_bundles(&definition, false)
            .into_iter()
            .map(|(pbr, _)| pbr)
            .collect::<Vec<_>>();
        if !pbr_bundles.is_empty() {
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
                .with_children(|child_builder| {
                    for pbr_bundle in pbr_bundles {
                        child_builder.spawn(pbr_bundle).insert(*child_visibiltiy);
                    }
                });
        }

        self.zone_level_entities.add(zone_level, entity.id());
        Ok(())
    }
}
