use crate::application::ApplicationState;
use crate::gameplay::{
    Accessible, ActiveSav, Amount, Aquatic, BaseSpeed, BodyContainers, CameraBase, Closeable,
    Containable, Craft, ExamineCursor, Explored, Faction, Filthy, HealingDuration, Health, Hurdle,
    Infos, ItemIntegrity, LastSeen, LevelOffset, Life, Limited, LocalTerrain, Melee, ModelFactory,
    ObjectCategory, ObjectName, Obstacle, Opaque, OpaqueFloor, Openable, Player, Pos, PosOffset,
    Shared, StairsDown, StairsUp, Stamina, StandardIntegrity, TileVariant, Vehicle, VehiclePart,
    WalkingMode, cdda::Error, item::Pocket, spawn::log_spawn_result,
};
use bevy::ecs::system::SystemParam;
use bevy::platform_support::collections::HashMap;
use bevy::prelude::{
    Camera3d, ChildOf, Commands, DirectionalLight, Entity, EulerRot, Mat4, Res, StateScoped,
    TextColor, Transform, Vec3, Visibility, debug, error,
};
use bevy::render::camera::{PerspectiveProjection, Projection};
use bevy::render::view::RenderLayers;
use cdda_json_files::{
    BashItem, BashItems, CddaAmount, CddaItem, CddaItemName, CddaVehicle, CddaVehiclePart,
    Character, CharacterInfo, CommonItemInfo, Field, Flags, FlatVec, FurnitureInfo, InfoId,
    ItemGroup, ItemName, MoveCostMod, PocketType, Recipe, Repetition, RequiredLinkedLater,
    SpawnItem, TerrainInfo, UntypedInfoId,
};
use either::Either;
use hud::{BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, WARN_TEXT_COLOR};
use std::sync::Arc;
use units::{Mass, Volume};
use util::here;

#[derive(SystemParam)]
pub(crate) struct TileSpawner<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    explored: Res<'w, Explored>,
    active_sav: Res<'w, ActiveSav>,
    model_factory: ModelFactory<'w>,
}

impl<'w> TileSpawner<'w, '_> {
    pub(crate) fn model_factory(&mut self) -> &mut ModelFactory<'w> {
        &mut self.model_factory
    }

    pub(crate) fn spawn_tile<'a>(
        &mut self,
        subzone_level_entity: Entity,
        pos: Pos,
        local_terrain: &LocalTerrain,
        furniture_infos: impl Iterator<Item = Arc<FurnitureInfo>>,
        item_repetitions: impl Iterator<Item = &'a Vec<Repetition<CddaItem>>>,
        spawns: impl Iterator<Item = &'a Character>,
        fields: impl Iterator<Item = &'a FlatVec<Field, 3>>,
    ) {
        self.spawn_terrain(subzone_level_entity, pos, local_terrain);

        for furniture_info in furniture_infos {
            self.spawn_furniture(subzone_level_entity, pos, &furniture_info);
        }

        for repetitions in item_repetitions {
            for repetition in repetitions {
                let CddaAmount { obj: item, amount } = repetition.as_amount();
                if let Err(error) = self.spawn_item(
                    subzone_level_entity,
                    Some(pos),
                    item,
                    Amount(item.charges.unwrap_or(1) * amount),
                ) {
                    error!("Spawning a tile item failed: {error:#?}");
                }
            }
        }

        for spawn in spawns {
            //trace!("{:?}", (&spawn.id);
            log_spawn_result(self.spawn_character(subzone_level_entity, pos, &spawn.info, None));
        }

        for fields in fields {
            //trace!("{:?}", (&fields);
            for field in &fields.0 {
                self.spawn_field(subzone_level_entity, pos, field);
            }
        }
    }

    pub(crate) fn spawn_character(
        &mut self,
        parent: Entity,
        pos: Pos,
        character_info: &RequiredLinkedLater<CharacterInfo>,
        name: Option<ObjectName>,
    ) -> Result<Entity, Error> {
        let character_info = character_info.get()?;
        let faction = match &*character_info.default_faction {
            "human" => Faction::Human,
            "zombie" => Faction::Zombie,
            _ => Faction::Animal,
        };
        let object_name = ObjectName::new(character_info.name.clone(), faction.color());

        let entity = self.spawn_object(
            parent,
            Some(pos),
            character_info.id.untyped(),
            ObjectCategory::Character,
            object_name,
            None,
        );
        let mut entity = self.commands.entity(entity);
        entity.insert((
            Shared::new(character_info.clone()),
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
                    ChildOf(entity),
                ))
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
                    ChildOf(entity),
                ))
                .id();
            self.commands
                .entity(entity)
                .insert(BodyContainers { hands, clothing });
        }

        debug!("Spawned a {:?} at {pos:?}", character_info.id);
        Ok(entity)
    }

    fn spawn_field(&mut self, parent: Entity, pos: Pos, field: &Field) {
        let Some(field_info) = field.field_info.get_option(here!()) else {
            return;
        };

        let object_name = ObjectName::new(field_info.name().clone(), BAD_TEXT_COLOR);

        let entity = self.spawn_object(
            parent,
            Some(pos),
            field_info.id.untyped(),
            ObjectCategory::Field,
            object_name,
            None,
        );
        self.commands.entity(entity).insert(Shared::new(field_info));
    }

    pub(crate) fn spawn_item(
        &mut self,
        parent: Entity,
        pos: Option<Pos>,
        item: &CddaItem,
        amount: Amount,
    ) -> Result<Entity, Error> {
        let item_info = &item.item_info.get()?;

        //trace!("{:?} {:?} {:?} {:?}", &parent, pos, &id, &amount);
        let object_name = ObjectName::new(
            item_info.name.clone(),
            item_category_text_color(item_info.category.as_ref()),
        );
        let object_entity = self.spawn_object(
            parent,
            pos,
            item_info.id.untyped(),
            ObjectCategory::Item,
            object_name,
            None,
        );

        let (volume, mass) = match &item.corpse.get() {
            Some(corpse_character) => {
                if corpse_character.id == InfoId::new("mon_null") {
                    (item_info.volume, item_info.mass)
                } else {
                    debug!("{:?}", &corpse_character.id);
                    (corpse_character.volume, corpse_character.mass)
                }
            }
            _ => (item_info.volume, item_info.mass),
        };

        let mut entity = self.commands.entity(object_entity);
        entity.insert((
            Shared::new(item_info.clone()),
            amount,
            Containable {
                // Based on cataclysm-ddasrc/mtype.cpp lines 47-48
                volume: volume
                    .unwrap_or_else(|| Volume::try_from("62499 ml").expect("Well formatted")),
                mass: mass.unwrap_or_else(|| Mass::try_from("81499 g").expect("Well formatted")),
            },
            ItemIntegrity::from(item.damaged),
        ));

        if item.item_tags.contains(&Arc::from("FILTHY")) {
            entity.insert(Filthy);
        }

        let entity = entity.id();
        //trace!("Item {entity:?} with parent {parent:?}");
        if let Some(container) = &item.contents {
            for cdda_pocket in &container.contents {
                //trace!("Pocket of {:?}: {:?}", &item.typeid, cdda_pocket);
                let pocket = self
                    .commands
                    .spawn((
                        Pocket::from(cdda_pocket),
                        Visibility::Hidden,
                        Transform::IDENTITY,
                        ChildOf(entity),
                    ))
                    .id();
                //trace!("Pocket {pocket:?} with parent {entity:?}");
                for content in &cdda_pocket.contents {
                    if let Err(error) =
                        self.spawn_item(pocket, None, content, Amount(content.charges.unwrap_or(1)))
                    {
                        error!("Spawning a nested item failed: {:#?}", error);
                    }
                }
            }
            // TODO container.additional_pockets
        }

        Ok(entity)
    }

    fn spawn_items(
        &mut self,
        parent_entity: Entity,
        pos: Pos,
        items: impl Iterator<Item = SpawnItem>,
    ) {
        for spawn_item in items {
            let mut cdda_item = CddaItem::from(&spawn_item.item_info);
            cdda_item.charges = spawn_item.charges;
            if let Err(error) = self.spawn_item(
                parent_entity,
                Some(pos),
                &cdda_item,
                Amount(spawn_item.amount),
            ) {
                error!("Spawning an item from a collection failed: {:#?}", error);
            }
        }
    }

    fn spawn_item_collection(
        &mut self,
        parent_entity: Entity,
        pos: Pos,
        item_group: &RequiredLinkedLater<ItemGroup>,
    ) {
        let Some(item_group) = item_group.get_option(here!()) else {
            return;
        };
        self.spawn_items(parent_entity, pos, item_group.items());
    }

    fn spawn_furniture(&mut self, parent: Entity, pos: Pos, furniture_info: &Arc<FurnitureInfo>) {
        let object_name = ObjectName::new(furniture_info.name.clone(), HARD_TEXT_COLOR);
        let entity = self.spawn_object(
            parent,
            Some(pos),
            furniture_info.id.untyped(),
            ObjectCategory::Furniture,
            object_name,
            Some(TileVariant::Unconnected),
        );
        let mut entity = self.commands.entity(entity);
        entity.insert(Shared::new(furniture_info.clone()));

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

    pub(crate) fn spawn_terrain(&mut self, parent: Entity, pos: Pos, local_terrain: &LocalTerrain) {
        if local_terrain.info.id == InfoId::new("t_open_air")
            || local_terrain.info.id == InfoId::new("t_open_air_rooved")
        {
            // Don't spawn air terrain to keep the entity count low
            return;
        }

        let object_name = ObjectName::new(local_terrain.info.name.clone(), HARD_TEXT_COLOR);
        let entity = self.spawn_object(
            parent,
            Some(pos),
            local_terrain.info.id.untyped(),
            ObjectCategory::Terrain,
            object_name,
            Some(local_terrain.variant),
        );
        let mut entity = self.commands.entity(entity);
        entity.insert(Shared::new(local_terrain.info.clone()));

        if local_terrain.info.move_cost.accessible() {
            if local_terrain.info.close.get().is_some() {
                entity.insert(Closeable);
            }
            entity.insert(Accessible {
                water: local_terrain.info.flags.water(),
                move_cost: local_terrain.info.move_cost,
            });
        } else if local_terrain.info.open.get().is_some() {
            entity.insert(Openable);
        } else {
            entity.insert(Obstacle);
        }

        if !local_terrain.info.flags.transparent() {
            entity.insert(Opaque);
        }

        if local_terrain.info.flags.goes_up() {
            entity.insert(StairsUp);
        }

        if local_terrain.info.flags.goes_down() {
            entity.insert(StairsDown);
        } else {
            entity.insert(OpaqueFloor);
        }

        if let Some(ref bash) = local_terrain.info.bash {
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
        let object_name = ObjectName::from_str(&vehicle.name, HARD_TEXT_COLOR);

        let entity = self.spawn_object(
            parent,
            Some(pos),
            &vehicle.id,
            ObjectCategory::Vehicle,
            object_name,
            None,
        );
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
        parent: Entity,
        parent_pos: Pos,
        vehicle_part: &CddaVehiclePart,
    ) {
        let Some(part_info) = vehicle_part.info.get_option(here!()) else {
            return;
        };
        let Some(item_info) = part_info.item.get_option(here!()) else {
            return;
        };

        let name = part_info.name.as_ref().unwrap_or(&item_info.name);
        let pos = parent_pos.horizontal_offset(vehicle_part.mount_dx, vehicle_part.mount_dy);
        let object_name = ObjectName::new(name.clone(), HARD_TEXT_COLOR);

        let variant = ItemIntegrity::from(vehicle_part.base.damaged)
            .broken()
            .then_some(TileVariant::Broken)
            .or_else(|| vehicle_part.open.then_some(TileVariant::Open))
            .unwrap_or(TileVariant::Unconnected);
        let entity = self.spawn_object(
            parent,
            Some(pos),
            part_info.id.untyped(),
            ObjectCategory::VehiclePart,
            object_name,
            Some(variant),
        );
        self.commands.entity(entity).insert((
            Shared::new(part_info.clone()),
            VehiclePart {
                offset: PosOffset {
                    x: vehicle_part.mount_dx,
                    level: LevelOffset::ZERO,
                    z: vehicle_part.mount_dy,
                },
                item: vehicle_part.base.clone(),
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
        info_id: &UntypedInfoId,
        category: ObjectCategory,
        object_name: ObjectName,
        tile_variant: Option<TileVariant>,
    ) -> Entity {
        //trace!("{:?} {pos:?} {:?} {:?} {:?}", &parent, &info_id, &category);
        let last_seen = if category.shading_applied() {
            if pos.is_some_and(|p| self.explored.has_pos_been_seen(p)) {
                LastSeen::Previously
            } else {
                LastSeen::Never
            }
        } else {
            // cursor -> dummy value that gives normal material
            LastSeen::Currently
        };
        //trace!("{:?}", &last_seen);

        let layers = if category == ObjectCategory::Vehicle {
            None
        } else {
            Some(
                self.model_factory
                    .get_layers(info_id, category, tile_variant)
                    .map(|(mesh, transform, apprearance)| {
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
                    }),
            )
        };

        let mut entity_commands = self.commands.spawn((
            object_name,
            Visibility::Hidden,
            Transform::IDENTITY,
            ChildOf(parent),
        ));
        if let Some(pos) = pos {
            entity_commands.insert((Transform::from_translation(pos.vec3()), pos));
        }
        if let Some(layers) = layers {
            entity_commands.with_children(|child_builder| {
                child_builder.spawn(layers.base);
                if let Some(overlay) = layers.overlay {
                    child_builder.spawn(overlay);
                }
            });
        }
        if category.shading_applied() {
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
        //trace!("{:?}", (&light_transform);
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

        let human = RequiredLinkedLater::from(InfoId::new("human"));
        infos.link_character(&human, "player");

        let sav = self.active_sav.sav();
        let player = self
            .spawn_character(
                root,
                spawn_pos.horizontal_offset(36, 56),
                &human,
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

        let human = RequiredLinkedLater::from(InfoId::new("human"));
        infos.link_character(&human, "survivor");

        log_spawn_result(self.spawn_character(
            root,
            around_pos.horizontal_offset(-26, -36),
            &human,
            Some(ObjectName::from_str("Survivor", HARD_TEXT_COLOR)),
        ));

        let zombie = RequiredLinkedLater::from(InfoId::new("mon_zombie"));
        infos.link_character(&zombie, "zombie");

        log_spawn_result(self.spawn_character(
            root,
            around_pos.horizontal_offset(-24, -30),
            &zombie,
            None,
        ));
        log_spawn_result(self.spawn_character(
            root,
            around_pos.horizontal_offset(4, -26),
            &zombie,
            None,
        ));
        log_spawn_result(self.spawn_character(
            root,
            around_pos.horizontal_offset(2, -27),
            &zombie,
            None,
        ));
        log_spawn_result(self.spawn_character(
            root,
            around_pos.horizontal_offset(1, -29),
            &zombie,
            None,
        ));
        log_spawn_result(self.spawn_character(
            root,
            around_pos.horizontal_offset(-2, -42),
            &zombie,
            None,
        ));
    }

    pub(crate) fn spawn_smashed(
        &mut self,
        parent_entity: Entity,
        pos: Pos,
        info: Either<&Arc<TerrainInfo>, &Arc<FurnitureInfo>>,
    ) {
        let Some(bash) = info.either(
            |terrain_info| terrain_info.bash.as_ref(),
            |furniture_info| furniture_info.bash.as_ref(),
        ) else {
            return;
        };

        if let Some(new_terrain) = &bash.terrain.get() {
            let local_terrain = LocalTerrain::unconnected(new_terrain.clone());
            self.spawn_terrain(parent_entity, pos, &local_terrain);
        }

        if let Some(furniture_id) = &bash.furniture.get() {
            self.spawn_furniture(parent_entity, pos, furniture_id);
        }

        if let Some(items) = &bash.items {
            match items {
                BashItems::Explicit(item_vec) => {
                    self.spawn_items(
                        parent_entity,
                        pos,
                        item_vec.iter().flat_map(BashItem::items),
                    );
                }
                BashItems::Collection(item_group) => {
                    self.spawn_item_collection(parent_entity, pos, item_group);
                }
            }
        }
    }

    pub(crate) fn spawn_craft(
        &mut self,
        parent_entity: Entity,
        pos: Pos,
        recipe: Arc<Recipe>,
    ) -> Result<Entity, Error> {
        let craft_item_info = CommonItemInfo {
            id: InfoId::new("craft"),
            category: None,
            proportional: None,
            relative: None,
            count: None,
            stack_size: None,
            range: None,
            dispersion: None,
            recoil: None,
            loudness: None,
            mass: None,
            integral_mass: None,
            volume: None,
            longest_side: None,
            price: None,
            price_postapoc: None,
            integral_volume: None,
            integral_longest_side: None,
            bashing: None,
            cutting: None,
            to_hit: None,
            variant_type: None,
            variants: None,
            container: None,
            sealed: None,
            emits: None,
            explode_in_fire: None,
            solar_efficiency: None,
            ascii_picture: None,
            thrown_damage: None,
            repairs_like: None,
            weapon_category: None,
            degradation_multiplier: None,
            name: ItemName::from(CddaItemName::Simple(String::from("Craft").into())),
            description: None,
            symbol: None,
            color: None,
            material: None,
            material_thickness: None,
            chat_topics: None,
            phase: None,
            magazines: None,
            min_skills: None,
            explosion: None,
            flags: Flags::default(),
            faults: None,
            qualities: Vec::new(),
            extend: None,
            delete: None,
            properties: None,
            techniques: None,
            max_charges: None,
            initial_charges: None,
            use_action: None,
            countdown_interval: None,
            countdown_destroy: None,
            countdown_action: None,
            looks_like: None,
            conditional_names: None,
            armor_data: None,
            pet_armor_data: None,
            gun_data: None,
            bionic_data: None,
            seed_data: None,
            relic_data: None,
            milling: None,
            gunmod_data: None,
            pocket_data: None,
            armor: None,
            snippet_category: None,
            extra: HashMap::default(),
        };
        let entity = self.spawn_item(
            parent_entity,
            Some(pos),
            &CddaItem::from(&Arc::new(craft_item_info)),
            Amount::SINGLE,
        )?;
        let crafting_time = recipe.time.ok_or_else(|| Error::RecipeWithoutTime {
            _id: recipe.id.clone(),
        })?;
        let craft = Craft::new(recipe, crafting_time);
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
