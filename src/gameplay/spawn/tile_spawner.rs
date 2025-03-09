use crate::application::ApplicationState;
use crate::gameplay::{
    Accessible, ActiveSav, Amount, Aquatic, BaseSpeed, BodyContainers, CameraBase, Closeable,
    Containable, Craft, ExamineCursor, Explored, Faction, Filthy, HealingDuration, Health, Hurdle,
    Info, Infos, ItemIntegrity, LastSeen, LevelOffset, Life, Limited, LocalTerrain, Melee,
    ModelFactory, ObjectCategory, ObjectDefinition, ObjectName, Obstacle, Opaque, OpaqueFloor,
    Openable, Player, Pos, PosOffset, StairsDown, StairsUp, Stamina, StandardIntegrity,
    TileVariant, Vehicle, VehiclePart, WalkingMode, cdda::Error, item::Pocket,
    spawn::log_spawn_result,
};
use crate::hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, WARN_TEXT_COLOR};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    BuildChildren as _, Camera3d, ChildBuild as _, Commands, DirectionalLight, Entity, EulerRot,
    Mat4, Res, StateScoped, TextColor, Transform, Vec3, Visibility,
};
use bevy::render::camera::{PerspectiveProjection, Projection};
use bevy::render::view::RenderLayers;
use cdda_json_files::{
    Bash, BashItem, BashItems, CddaAmount, CddaItem, CddaVehicle, CddaVehiclePart, CountRange,
    Field, FlatVec, MoveCostMod, ObjectId, PocketType, Repetition, Spawn, TerrainInfo,
};
use std::sync::Arc;
use units::{Mass, Volume};

#[derive(SystemParam)]
pub(crate) struct TileSpawner<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) explored: Res<'w, Explored>,
    active_sav: Res<'w, ActiveSav>,
    model_factory: ModelFactory<'w>,
}

impl<'w> TileSpawner<'w, '_> {
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
        let terrain_info = match infos.terrain(&local_terrain.id) {
            Ok(terrain_info) => terrain_info,
            Err(error) => {
                dbg!(error);
                return;
            }
        };
        self.spawn_terrain(terrain_info, subzone_level_entity, pos, local_terrain);

        for id in furniture_ids {
            self.spawn_furniture(infos, subzone_level_entity, pos, id);
        }

        for repetitions in item_repetitions {
            for repetition in repetitions {
                let CddaAmount { obj: item, amount } = repetition.as_amount();
                _ = self.spawn_item(
                    infos,
                    subzone_level_entity,
                    Some(pos),
                    item,
                    Amount(item.charges.unwrap_or(1) * amount),
                );
            }
        }

        for spawn in spawns {
            //dbg!(&spawn.id);
            log_spawn_result(self.spawn_character(
                infos,
                subzone_level_entity,
                pos,
                &spawn.id,
                None,
            ));
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
    ) -> Result<Entity, Error> {
        let character_info = infos.character(id)?;
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
        let entity = self.spawn_object(parent, Some(pos), definition, object_name, None);
        let mut entity = self.commands.entity(entity);
        entity.insert((
            Info::new(character_info.clone()),
            Life,
            Obstacle,
            Health(Limited::full(character_info.hp.unwrap_or(60) as u16)),
            Stamina::Unlimited,
            WalkingMode::Perpetual,
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
                    Transform::default(),
                    Visibility::Hidden,
                    Pocket {
                        type_: PocketType::Container,
                        sealed: false,
                    },
                ))
                .set_parent(entity)
                .id();
            let clothing = self
                .commands
                .spawn((
                    BodyContainers::default_clothing_container_limits(),
                    Transform::default(),
                    Visibility::Hidden,
                    Pocket {
                        type_: PocketType::Container,
                        sealed: false,
                    },
                ))
                .set_parent(entity)
                .id();
            self.commands
                .entity(entity)
                .insert(BodyContainers { hands, clothing });
        }

        println!("Spawned a {:?} at {pos:?}", definition.id);
        Ok(entity)
    }

    fn spawn_field(&mut self, infos: &Infos, parent: Entity, pos: Pos, id: &ObjectId) {
        let field_info = match infos.field(id) {
            Ok(field_info) => field_info,
            Err(error) => {
                dbg!(error);
                return;
            }
        };
        let object_name = ObjectName::new(field_info.name().clone(), BAD_TEXT_COLOR);

        let definition = &ObjectDefinition {
            category: ObjectCategory::Field,
            id: id.clone(),
        };
        let entity = self.spawn_object(parent, Some(pos), definition, object_name, None);
        self.commands
            .entity(entity)
            .insert(Info::new(field_info.clone()));
    }

    pub(crate) fn spawn_item(
        &mut self,
        infos: &Infos,
        parent: Entity,
        pos: Option<Pos>,
        item: &CddaItem,
        amount: Amount,
    ) -> Result<Entity, Error> {
        let item_info = infos.common_item_info(&item.typeid)?;

        //println!("{:?} {:?} {:?} {:?}", &parent, pos, &id, &amount);
        let definition = &ObjectDefinition {
            category: ObjectCategory::Item,
            id: item.typeid.clone(),
        };
        //println!("{:?} @ {pos:?}", &definition);
        let object_name = ObjectName::new(
            item_info.name.clone(),
            item_category_text_color(item_info.category.as_ref()),
        );
        let object_entity = self.spawn_object(parent, pos, definition, object_name, None);

        let (volume, mass) = match &item.corpse {
            Some(corpse_id) if corpse_id != &ObjectId::new("mon_null") => {
                println!("{:?}", &corpse_id);
                match infos.character(corpse_id) {
                    Ok(monster_info) => (monster_info.volume, monster_info.mass),
                    Err(_) => (item_info.volume, item_info.mass),
                }
            }
            _ => (item_info.volume, item_info.mass),
        };

        let mut entity = self.commands.entity(object_entity);
        entity.insert((
            Info::new(item_info.clone()),
            amount,
            Containable {
                // Based on cataclysm-ddasrc/mtype.cpp lines 47-48
                volume: volume.unwrap_or_else(|| Volume::from("62499 ml")),
                mass: mass.unwrap_or_else(|| Mass::from("81499 g")),
            },
            ItemIntegrity::from(item.damaged),
        ));

        if item.item_tags.contains(&Arc::from("FILTHY")) {
            entity.insert(Filthy);
        }

        let entity = entity.id();
        //println!("Item {entity:?} with parent {parent:?}");
        if let Some(container) = &item.contents {
            for cdda_pocket in &container.contents {
                //println!("Pocket of {:?}: {:?}", &item.typeid, cdda_pocket);
                let pocket = self
                    .commands
                    .spawn((
                        Pocket::from(cdda_pocket),
                        Visibility::Hidden,
                        Transform::IDENTITY,
                    ))
                    .set_parent(entity)
                    .id();
                //println!("Pocket {pocket:?} with parent {entity:?}");
                for content in &cdda_pocket.contents {
                    let result = self.spawn_item(
                        infos,
                        pocket,
                        None,
                        content,
                        Amount(content.charges.unwrap_or(1)),
                    );
                    if let Err(error) = result {
                        dbg!(error);
                    }
                }
            }
            // TODO container.additional_pockets
        }

        Ok(entity)
    }

    fn spawn_bashing_items(
        &mut self,
        infos: &Infos,
        parent_entity: Entity,
        pos: Pos,
        items: &Vec<BashItem>,
    ) {
        for item in items {
            match *item {
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
                        _ = self.spawn_item(infos, parent_entity, Some(pos), &cdda_item, amount);
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
        let item_group = match infos.item_group(id) {
            Ok(item_group) => item_group,
            Err(error) => {
                dbg!(error);
                return;
            }
        };
        let items = item_group
            .items
            .as_ref()
            .or(item_group.entries.as_ref())
            .expect("items or entries");
        self.spawn_bashing_items(infos, parent_entity, pos, items);
    }

    fn spawn_furniture(&mut self, infos: &Infos, parent: Entity, pos: Pos, id: &ObjectId) {
        let furniture_info = match infos.furniture(id) {
            Ok(furniture_info) => furniture_info,
            Err(error) => {
                dbg!(error);
                return;
            }
        };

        let definition = &ObjectDefinition {
            category: ObjectCategory::Furniture,
            id: id.clone(),
        };
        let object_name = ObjectName::new(furniture_info.name.clone(), HARD_TEXT_COLOR);
        let entity = self.spawn_object(parent, Some(pos), definition, object_name, None);
        let mut entity = self.commands.entity(entity);
        entity.insert(Info::new(furniture_info.clone()));

        if !furniture_info.flags.transparent() {
            entity.insert(Opaque);
        }

        if furniture_info.bash.is_some() {
            // TODO
            entity.insert(StandardIntegrity(Limited::full(10)));
        }

        match furniture_info.move_cost_mod {
            None => {}
            Some(MoveCostMod::Impossible(_)) => {
                entity.insert(Obstacle);
            }
            Some(MoveCostMod::Slower(increase)) => {
                entity.insert(Hurdle(increase));
            }
        }
    }

    pub(crate) fn spawn_terrain(
        &mut self,
        terrain_info: &Arc<TerrainInfo>,
        parent: Entity,
        pos: Pos,
        local_terrain: &LocalTerrain,
    ) {
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
        let entity = self.spawn_object(
            parent,
            Some(pos),
            definition,
            object_name,
            Some(local_terrain.variant),
        );
        let mut entity = self.commands.entity(entity);
        entity.insert(Info::new(terrain_info.clone()));

        if terrain_info.move_cost.accessible() {
            if terrain_info.close.get().is_some() {
                entity.insert(Closeable);
            }
            entity.insert(Accessible {
                water: terrain_info.flags.water(),
                move_cost: terrain_info.move_cost,
            });
        } else if terrain_info.open.get().is_some() {
            entity.insert(Openable);
        } else {
            entity.insert(Obstacle);
        }

        if !terrain_info.flags.transparent() {
            entity.insert(Opaque);
        }

        if terrain_info.flags.goes_up() {
            entity.insert(StairsUp);
        }

        if terrain_info.flags.goes_down() {
            entity.insert(StairsDown);
        } else {
            entity.insert(OpaqueFloor);
        }

        if let Some(ref bash) = terrain_info.bash {
            if let Some(_new_terrain) = bash.terrain.get() {
                entity.insert(StandardIntegrity(Limited::full(10)));
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

        let entity = self.spawn_object(parent, Some(pos), definition, object_name, None);
        self.commands.entity(entity).insert((
            Vehicle,
            pos,
            Transform::IDENTITY,
            Visibility::Inherited,
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
        let part_info = match infos.vehicle_part(&part.id) {
            Ok(part_info) => part_info,
            Err(error) => {
                dbg!(error);
                return;
            }
        };
        let item_info = match infos.common_item_info(&part_info.item) {
            Ok(item_info) => item_info,
            Err(error) => {
                dbg!(error);
                return;
            }
        };

        let name = part_info.name.as_ref().unwrap_or(&item_info.name);
        let pos = parent_pos.horizontal_offset(part.mount_dx, part.mount_dy);
        let definition = &ObjectDefinition {
            category: ObjectCategory::VehiclePart,
            id: part.id.clone(),
        };
        let object_name = ObjectName::new(name.clone(), HARD_TEXT_COLOR);

        let variant = ItemIntegrity::from(part.base.damaged)
            .broken()
            .then_some(TileVariant::Broken)
            .or_else(|| part.open.then_some(TileVariant::Open));
        let entity = self.spawn_object(parent, Some(pos), definition, object_name, variant);
        self.commands.entity(entity).insert((
            Info::new(part_info.clone()),
            VehiclePart {
                offset: PosOffset {
                    x: part.mount_dx,
                    level: LevelOffset::ZERO,
                    z: part.mount_dy,
                },
                item: part.base.clone(),
            },
        ));

        if part_info.flags.obstacle() {
            self.commands.entity(entity).insert(Obstacle);
        }
    }

    fn spawn_object(
        &mut self,
        parent: Entity,
        pos: Option<Pos>,
        definition: &ObjectDefinition,
        object_name: ObjectName,
        tile_variant: Option<TileVariant>,
    ) -> Entity {
        //dbg!(&parent);
        //dbg!(pos);
        //dbg!(&definition);
        let last_seen = if definition.category.shading_applied() {
            if pos.is_some_and(|p| self.explored.has_pos_been_seen(p)) {
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
            Some(self.model_factory.get_layers(definition, tile_variant).map(
                |(mesh, transform, apprearance)| {
                    let material = if last_seen == LastSeen::Never {
                        apprearance.material(&LastSeen::Currently)
                    } else {
                        apprearance.material(&last_seen)
                    };
                    (
                        mesh,
                        material,
                        transform,
                        apprearance,
                        Visibility::Inherited,
                        RenderLayers::layer(1),
                    )
                },
            ))
        };

        let mut entity_commands = self.commands.spawn((
            definition.clone(),
            object_name,
            Visibility::Hidden,
            Transform::IDENTITY,
        ));
        if let Some(pos) = pos {
            entity_commands.insert((Transform::from_translation(pos.vec3()), pos));
        }
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
        self.commands
            .entity(player_entity)
            .with_children(|child_builder| {
                child_builder
                    .spawn((Transform::default(), Visibility::default(), CameraBase))
                    .with_children(|child_builder| {
                        let cursor_bundle = self.model_factory.get_cursor();
                        child_builder.spawn((cursor_bundle, Visibility::Hidden, ExamineCursor));

                        let camera_direction = Transform::IDENTITY
                            .looking_at(Vec3::new(0.1, 0.0, -1.0), Vec3::Y)
                            * Transform::from_translation(Vec3::new(0.0, 0.3, 0.0));
                        child_builder
                            .spawn((camera_direction, Visibility::Hidden))
                            .with_children(|child_builder| {
                                child_builder.spawn((
                                    Camera3d::default(),
                                    Projection::Perspective(PerspectiveProjection {
                                        // more overview, less personal than the default
                                        fov: 0.3,
                                        ..PerspectiveProjection::default()
                                    }),
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
            DirectionalLight {
                illuminance: 10_000.0,
                shadows_enabled: false, // TODO shadow direction does not match buildin shadows
                ..DirectionalLight::default()
            },
            light_transform,
            StateScoped(ApplicationState::Gameplay),
        ));
    }

    pub(crate) fn spawn_characters(&mut self, infos: &Infos, spawn_pos: Pos) {
        let root = self
            .commands
            .spawn((
                Transform::default(),
                Visibility::default(),
                StateScoped(ApplicationState::Gameplay),
            ))
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
            .insert(Stamina::FULL)
            .insert(WalkingMode::Walking); // override
        self.configure_player(player);
    }

    pub(crate) fn spawn_zombies(&mut self, infos: &Infos, around_pos: Pos) {
        let root = self
            .commands
            .spawn((
                Transform::default(),
                Visibility::default(),
                StateScoped(ApplicationState::Gameplay),
            ))
            .id();

        log_spawn_result(self.spawn_character(
            infos,
            root,
            around_pos.horizontal_offset(-26, -36),
            &ObjectId::new("human"),
            Some(ObjectName::from_str("Survivor", HARD_TEXT_COLOR)),
        ));

        log_spawn_result(self.spawn_character(
            infos,
            root,
            around_pos.horizontal_offset(-24, -30),
            &ObjectId::new("mon_zombie"),
            None,
        ));
        log_spawn_result(self.spawn_character(
            infos,
            root,
            around_pos.horizontal_offset(4, -26),
            &ObjectId::new("mon_zombie"),
            None,
        ));
        log_spawn_result(self.spawn_character(
            infos,
            root,
            around_pos.horizontal_offset(2, -27),
            &ObjectId::new("mon_zombie"),
            None,
        ));
        log_spawn_result(self.spawn_character(
            infos,
            root,
            around_pos.horizontal_offset(1, -29),
            &ObjectId::new("mon_zombie"),
            None,
        ));
        log_spawn_result(self.spawn_character(
            infos,
            root,
            around_pos.horizontal_offset(-2, -42),
            &ObjectId::new("mon_zombie"),
            None,
        ));
    }

    pub(crate) fn spawn_smashed(
        &mut self,
        infos: &Infos,
        parent_entity: Entity,
        pos: Pos,
        definition: &ObjectDefinition,
        bash: &Bash,
    ) {
        assert!(
            matches!(
                definition.category,
                ObjectCategory::Furniture | ObjectCategory::Terrain
            ),
            "Only furniture and terrain can be smashed"
        );

        if let Some(new_terrain) = &bash.terrain.get() {
            assert_eq!(
                definition.category,
                ObjectCategory::Terrain,
                "The terrain field requires a terrain category"
            );
            let local_terrain = LocalTerrain::unconnected(new_terrain.id.clone());
            self.spawn_terrain(new_terrain, parent_entity, pos, &local_terrain);
        } else if let Some(furniture_id) = &bash.furniture {
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
    ) -> Result<Entity, Error> {
        let craft = CddaItem::from(ObjectId::new("craft"));
        let entity = self.spawn_item(infos, parent_entity, Some(pos), &craft, Amount::SINGLE)?;

        let recipe = infos.recipe(&recipe_id)?;
        let crafting_time = recipe.time.ok_or_else(|| Error::RecipeWithoutTime {
            _id: recipe_id.clone(),
        })?;
        let craft = Craft::new(recipe_id, crafting_time);
        self.commands.entity(entity).insert(craft);

        Ok(entity)
    }
}

fn item_category_text_color(from: Option<&Arc<str>>) -> TextColor {
    if from == Some(&Arc::from("manuals")) {
        GOOD_TEXT_COLOR
    } else if from == Some(&Arc::from("bionics")) {
        WARN_TEXT_COLOR
    } else {
        HARD_TEXT_COLOR
    }
}
