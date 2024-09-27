use crate::application::ApplicationState;
use crate::gameplay::{
    Accessible, ActiveSav, Amount, Aquatic, BaseSpeed, BodyContainers, CameraBase, Closeable,
    Containable, Craft, ExamineCursor, Explored, Faction, Filthy, HealingDuration, Health, Hurdle,
    Infos, Integrity, LastSeen, LevelOffset, Life, Limited, LocalTerrain, Melee, ModelFactory,
    ObjectCategory, ObjectDefinition, ObjectName, Obstacle, Opaque, OpaqueFloor, Openable, Player,
    Pos, PosOffset, StairsDown, StairsUp, Stamina, TileVariant, Vehicle, VehiclePart, WalkingMode,
};
use crate::hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, WARN_TEXT_COLOR};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    BuildChildren, Camera3dBundle, Color, Commands, DirectionalLight, DirectionalLightBundle,
    Entity, EulerRot, Mat4, PbrBundle, Res, ResMut, SpatialBundle, StateScoped, Transform, Vec3,
    Visibility,
};
use bevy::render::camera::{PerspectiveProjection, Projection};
use bevy::render::view::RenderLayers;
use cdda_json_files::{
    BashItem, BashItems, CddaAmount, CddaItem, CddaVehicle, CddaVehiclePart, CountRange, Field,
    FlatVec, MoveCostMod, ObjectId, Repetition, Spawn,
};
use std::sync::Arc;
use units::{Mass, Volume};

#[derive(SystemParam)]
pub(crate) struct TileSpawner<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) explored: ResMut<'w, Explored>,
    active_sav: Res<'w, ActiveSav>,
    model_factory: ModelFactory<'w>,
}

impl<'w, 's> TileSpawner<'w, 's> {
    pub(crate) fn model_factory(&mut self) -> &mut ModelFactory<'w> {
        &mut self.model_factory
    }

    pub(crate) fn spawn_tile<'a>(
        &mut self,
        infos: &Infos,
        subzone_level_entity: Entity,
        pos: Pos,
        local_terrain: &LocalTerrain,
        furniture_ids: impl Iterator<Item = &'a ObjectId>,
        item_repetitions: impl Iterator<Item = &'a Vec<Repetition<CddaItem>>>,
        spawns: impl Iterator<Item = &'a Spawn>,
        fields: impl Iterator<Item = &'a FlatVec<Field, 3>>,
    ) {
        self.spawn_terrain(infos, subzone_level_entity, pos, local_terrain);

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

    pub(crate) fn spawn_character(
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
        let faction = match &*character_info.default_faction {
            "human" => Faction::Human,
            "zombie" => Faction::Zombie,
            _ => Faction::Animal,
        };
        let object_name = ObjectName::new(character_info.name.clone(), faction.color());

        let definition = &ObjectDefinition {
            category: ObjectCategory::Character,
            id: id.clone(),
        };
        let entity = self.spawn_object(parent, pos, definition, object_name, None);
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
        self.spawn_object(parent, pos, definition, object_name, None);
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
        let object_name = ObjectName::new(
            item_info.name.clone(),
            item_category_text_color(&item_info.category),
        );
        let object_entity = self.spawn_object(parent, pos, definition, object_name, None);

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

        if item.item_tags.contains(&Arc::from("FILTHY")) {
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
            eprintln!("No info found for {id:?}. Spawning skipped");
            return;
        };

        let definition = &ObjectDefinition {
            category: ObjectCategory::Furniture,
            id: id.clone(),
        };
        let object_name = ObjectName::new(furniture_info.name.clone(), HARD_TEXT_COLOR);
        let object_entity = self.spawn_object(parent, pos, definition, object_name, None);

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

    pub(crate) fn spawn_terrain(
        &mut self,
        infos: &Infos,
        parent: Entity,
        pos: Pos,
        local_terrain: &LocalTerrain,
    ) {
        let Some(terrain_info) = infos.try_terrain(&local_terrain.id) else {
            eprintln!("No info found for terrain {:?}", local_terrain.id);
            return;
        };

        if local_terrain.id == ObjectId::new("t_open_air")
            || local_terrain.id == ObjectId::new("t_open_air_rooved")
        {
            // Don't spawn air terrain to keep the entity count low
            return;
        }

        let definition = &ObjectDefinition {
            category: ObjectCategory::Terrain,
            id: local_terrain.id.clone(),
        };
        let object_name = ObjectName::new(terrain_info.name.clone(), HARD_TEXT_COLOR);
        let object_entity = self.spawn_object(
            parent,
            pos,
            definition,
            object_name,
            Some(local_terrain.variant),
        );

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

    pub(crate) fn spawn_vehicle(
        &mut self,
        parent: Entity,
        pos: Pos,
        vehicle: &CddaVehicle,
    ) -> Entity {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Vehicle,
            id: vehicle.id.clone(),
        };
        let object_name = ObjectName::from_str(&vehicle.name, HARD_TEXT_COLOR);

        let entity = self.spawn_object(parent, pos, definition, object_name, None);
        self.commands.entity(entity).insert((
            Vehicle,
            pos,
            Transform::IDENTITY,
            StateScoped(ApplicationState::Gameplay),
        ));

        entity
    }

    pub(crate) fn spawn_vehicle_part(
        &mut self,
        infos: &Infos,
        parent: Entity,
        parent_pos: Pos,
        part: &CddaVehiclePart,
    ) {
        let Some(part_info) = infos.try_vehicle_part(&part.id) else {
            eprintln!(
                "No info found for vehicle part {:?}. Spawning skipped",
                &part.id
            );
            return;
        };
        let Some(item_info) = infos.try_item(&part_info.item) else {
            eprintln!(
                "No info found for item {:?} from vehicle part {:?}. Spawning skipped",
                &part_info.item, &part.id
            );
            return;
        };

        let name = part_info.name.as_ref().unwrap_or(&item_info.name);
        let pos = parent_pos.horizontal_offset(part.mount_dx, part.mount_dy);
        let definition = &ObjectDefinition {
            category: ObjectCategory::VehiclePart,
            id: part.id.clone(),
        };
        let object_name = ObjectName::new(name.clone(), HARD_TEXT_COLOR);

        let variant = part
            .base
            .broken()
            .then_some(TileVariant::Broken)
            .or_else(|| part.open.then_some(TileVariant::Open));
        let entity = self.spawn_object(parent, pos, definition, object_name, variant);
        self.commands.entity(entity).insert(VehiclePart {
            offset: PosOffset {
                x: part.mount_dx,
                level: LevelOffset::ZERO,
                z: part.mount_dy,
            },
            item: part.base.clone(),
        });

        if part_info.flags.obstacle() {
            self.commands.entity(entity).insert(Obstacle);
        }
    }

    fn spawn_object(
        &mut self,
        parent: Entity,
        pos: Pos,
        definition: &ObjectDefinition,
        object_name: ObjectName,
        tile_variant: Option<TileVariant>,
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

        let layers = if definition.category == ObjectCategory::Vehicle {
            None
        } else {
            Some(
                self.model_factory
                    .get_layers(definition, Visibility::Inherited, true, tile_variant)
                    .map(|(mut pbr_bundle, apprearance)| {
                        pbr_bundle.material = if last_seen == LastSeen::Never {
                            apprearance.material(&LastSeen::Currently)
                        } else {
                            apprearance.material(&last_seen)
                        };
                        (pbr_bundle, apprearance, RenderLayers::layer(1))
                    }),
            )
        };

        let mut entity_commands = self.commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(pos.vec3()),
                ..SpatialBundle::HIDDEN_IDENTITY
            },
            definition.clone(),
            pos,
            object_name,
        ));
        entity_commands.set_parent(parent);
        if let Some(layers) = layers {
            entity_commands.with_children(|child_builder| {
                child_builder.spawn(layers.base);
                if let Some(overlay) = layers.overlay {
                    child_builder.spawn(overlay);
                }
            });
        }
        if definition.category.shading_applied() {
            entity_commands.insert(last_seen);
        }

        entity_commands.id()
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
                                        projection: Projection::Perspective(
                                            PerspectiveProjection {
                                                // more overview, less personal than the default
                                                fov: 0.3,
                                                ..PerspectiveProjection::default()
                                            },
                                        ),
                                        ..Camera3dBundle::default()
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

        let sav = self.active_sav.sav();
        let player = self
            .spawn_character(
                infos,
                root,
                spawn_pos.horizontal_offset(36, 56),
                &ObjectId::new("human"),
                Some(ObjectName::from_str(&sav.player.name, GOOD_TEXT_COLOR)),
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
            Some(ObjectName::from_str("Survivor", HARD_TEXT_COLOR)),
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
            let local_terrain = LocalTerrain::unconnected(terrain_id.clone());
            self.spawn_terrain(infos, parent_entity, pos, &local_terrain);
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
        recipe_id: ObjectId,
    ) -> Entity {
        let craft = CddaItem::from(ObjectId::new("craft"));
        let entity = self
            .spawn_item(infos, parent_entity, pos, &craft, Amount::SINGLE)
            .expect("Spawning craft item should have succeeded");

        let recipe = infos.recipe(&recipe_id);
        let crafting_time = recipe.time.expect("Craftable recipes should have a time");
        let craft = Craft::new(recipe_id, crafting_time);
        self.commands.entity(entity).insert(craft);

        entity
    }
}

fn item_category_text_color(from: &Option<Arc<str>>) -> Color {
    if from == &Some(Arc::from("manuals")) {
        GOOD_TEXT_COLOR
    } else if from == &Some(Arc::from("bionics")) {
        WARN_TEXT_COLOR
    } else {
        HARD_TEXT_COLOR
    }
}
