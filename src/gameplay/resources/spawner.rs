use crate::prelude::*;
use bevy::{
    ecs::system::{Insert, SystemParam},
    prelude::*,
    render::camera::{PerspectiveProjection, Projection::Perspective},
    utils::HashMap,
};
use std::{cmp::Ordering, path::PathBuf};

fn insert<T>(child_builder: &mut ChildBuilder, entity: Entity, bundle: T)
where
    T: Bundle + 'static,
{
    child_builder.add_command(Insert { entity, bundle });
}

#[derive(Default, Resource)]
pub(crate) struct TileCaches {
    appearance_cache: HashMap<PathBuf, Appearance>,
    horizontal_plane_mesh_cache: HashMap<SpriteNumber, Handle<Mesh>>,
    vertical_plane_mesh_cache: HashMap<SpriteNumber, Handle<Mesh>>,
    cuboid_mesh_cache: HashMap<SpriteNumber, Handle<Mesh>>,
}

#[derive(Default, Resource)]
pub(crate) struct Maps {
    pub(crate) loading: Vec<Handle<Map>>,
}

#[derive(SystemParam)]
pub(crate) struct Spawner<'w, 's> {
    pub(crate) commands: Commands<'w, 's>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    pub(crate) asset_server: Res<'w, AssetServer>,
    loader: Res<'w, TileLoader>,
    pub(crate) maps: ResMut<'w, Maps>,
    caches: ResMut<'w, TileCaches>,
    location: ResMut<'w, Location>,
    pub(crate) zone_level_ids: ResMut<'w, ZoneLevelIds>,
    pub(crate) zone_level_entities: ResMut<'w, ZoneLevelEntities>,
    pub(crate) subzone_level_entities: ResMut<'w, SubzoneLevelEntities>,
    pub(crate) explored: ResMut<'w, Explored>,
    pub(crate) infos: ResMut<'w, Infos>,
    paths: Res<'w, Paths>,
    sav: Res<'w, Sav>,
}

impl<'w, 's> Spawner<'w, 's> {
    fn get_mesh(&mut self, model: &Model) -> Handle<Mesh> {
        match model.shape {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                ..
            } => &mut self.caches.horizontal_plane_mesh_cache,
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                ..
            } => &mut self.caches.vertical_plane_mesh_cache,
            ModelShape::Cuboid { .. } => &mut self.caches.cuboid_mesh_cache,
        }
        .entry(model.sprite_number)
        .or_insert_with(|| self.mesh_assets.add(model.to_mesh()))
        .clone()
    }

    fn get_appearance(&mut self, model: &Model) -> Appearance {
        self.caches
            .appearance_cache
            .entry(model.texture_path.clone())
            .or_insert_with(|| {
                let material = StandardMaterial {
                    base_color_texture: Some(self.asset_server.load(model.texture_path.clone())),
                    alpha_mode: model.alpha_mode,
                    ..StandardMaterial::default()
                };
                Appearance::new(&mut self.material_assets, material)
            })
            .clone()
    }

    fn get_pbr_bundle(&mut self, model: &Model, shaded: bool) -> PbrBundle {
        PbrBundle {
            mesh: self.get_mesh(model),
            material: if shaded {
                Handle::<StandardMaterial>::default()
            } else {
                self.get_appearance(model).material(&LastSeen::Currently)
            },
            transform: model.to_transform(),
            visibility: if shaded {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            },
            ..PbrBundle::default()
        }
    }

    fn spawn_character(&mut self, parent: Entity, pos: Pos, id: ObjectId) -> Option<Entity> {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Character,
            id,
        };

        let entity = self.spawn_tile(parent, pos, definition);

        let Some(character_info) = self
            .infos
            .character(&definition.id) else {
                self.commands.entity(entity).despawn_recursive();
                println!("No info found for {:?}. Spawning skipped", definition.id);
                return None;
            };

        println!("Spawned a {:?} at {pos:?}", definition.id);

        self.commands
            .entity(entity)
            .insert(Obstacle)
            .insert(Health::new(character_info.hp.unwrap_or(60) as i16))
            .insert(match character_info.default_faction.as_str() {
                "human" => Faction::Human,
                "zombie" => Faction::Zombie,
                _ => Faction::Animal,
            })
            .insert(Hands::default())
            .insert(Clothing::default())
            .insert(Melee {
                dices: character_info.melee_dice,
                sides: character_info.melee_dice_sides,
            });

        if 0 < character_info.speed {
            self.commands
                .entity(entity)
                .insert(BaseSpeed::from_h_kmph(character_info.speed / 12));
        }

        if character_info.flags.aquatic() {
            self.commands.entity(entity).insert(Aquatic);
        }

        Some(entity)
    }

    fn spawn_item(
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
        let entity = self.spawn_tile(parent, pos, definition);

        let Some(item_info) = self
        .infos
        .item(&definition.id) else {
            self.commands.entity(entity).despawn_recursive();
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

        // Overwrite the default label
        self.commands
            .entity(entity)
            .insert(self.infos.label(definition, amount.0 as usize))
            .insert(amount)
            .insert(Containable {
                // Based on cataclysm-ddasrc/mtype.cpp lines 47-48
                volume: volume.unwrap_or_else(|| Volume::from(String::from("62499 ml"))),
                mass: mass.unwrap_or_else(|| Mass::from(String::from("81499 g"))),
            });
        Ok(())
    }

    fn spawn_furniture(&mut self, parent: Entity, pos: Pos, id: ObjectId) {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Furniture,
            id,
        };

        let tile = self.spawn_tile(parent, pos, definition);

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

        let tile = self.spawn_tile(parent, pos, definition);

        let Some(terrain_info) = self
        .infos
        .terrain(&definition.id) else {
            self.commands.entity(tile).despawn_recursive();
            println!("No info found for {:?}. Spawning skipped", definition.id);
            return;
        };

        match terrain_info {
            TerrainInfo::Terrain {
                move_cost,
                open,
                close,
                flags,
                ..
            } => {
                let move_cost = *move_cost;
                if 0 < move_cost.0 {
                    if close.is_some() {
                        self.commands.entity(tile).insert(Closeable);
                    }
                    self.commands.entity(tile).insert(Accessible {
                        water: flags.water(),
                        move_cost,
                    });
                } else if open.is_some() {
                    self.commands.entity(tile).insert(Openable);
                } else {
                    self.commands.entity(tile).insert(Obstacle);
                }

                if !flags.transparent() {
                    self.commands.entity(tile).insert(Opaque);
                }

                if flags.goes_up() {
                    self.commands.entity(tile).insert(StairsUp);
                }

                if flags.goes_down() {
                    self.commands.entity(tile).insert(StairsDown);
                } else {
                    self.commands.entity(tile).insert(OpaqueFloor);
                }
            }
            TerrainInfo::FieldType { .. } => {}
        }
    }

    fn spawn_tile(&mut self, parent: Entity, pos: Pos, definition: &ObjectDefinition) -> Entity {
        //dbg!(&parent);
        //dbg!(pos);
        //dbg!(&definition);
        let models = self
            .loader
            .get_models(definition, &self.infos.variants(definition));
        //dbg!(&models);
        let child_info = models
            .iter()
            .map(|model| (self.get_pbr_bundle(model, true), self.get_appearance(model)))
            .collect::<Vec<(PbrBundle, Appearance)>>();

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
            .insert(self.infos.label(definition, 1))
            .with_children(|child_builder| {
                for (pbr_bundle, apprearance) in child_info {
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
                        self.spawn_terrain(subzone_level_entity, pos, id.clone());
                    }

                    for id in submap
                        .furniture
                        .iter()
                        .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                    {
                        self.spawn_furniture(subzone_level_entity, pos, id.clone());
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
                            if let Err(load_error) = self.spawn_item(
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
                        self.spawn_character(subzone_level_entity, pos, spawn.id.clone());
                    }

                    for fields in submap
                        .fields
                        .0
                        .iter()
                        .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                    {
                        //dbg!(&fields);
                        for field in &fields.0 {
                            self.spawn_terrain(subzone_level_entity, pos, field.id.clone());
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
        assert!(zone_level.level <= Level::ZERO);

        let Some(seen_from) = self
            .explored
            .has_zone_level_been_seen(&self.asset_server, zone_level) else {
                return Err(());
        };

        let definition = ObjectDefinition {
            category: ObjectCategory::ZoneLevel,
            id: self.zone_level_ids.get(zone_level).clone(),
        };

        //println!("zone_level: {zone_level:?} {:?}", &definition);
        let pbr_bundles = self
            .loader
            .get_models(&definition, &self.infos.variants(&definition))
            .iter()
            .map(|model| self.get_pbr_bundle(model, false))
            .collect::<Vec<PbrBundle>>();

        let label = self.infos.label(&definition, 1);

        let mut entity = self.commands.spawn(zone_level);
        entity.insert(Collapsed).insert(label);

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

    fn configure_player(&mut self, player_entity: Entity) {
        let cursor_definition = ObjectDefinition {
            category: ObjectCategory::Meta,
            id: ObjectId::new("cursor"),
        };
        let cursor_model = &mut self
            .loader
            .get_models(&cursor_definition, &[cursor_definition.id.clone()])[0];
        let mut cursor_bundle = self.get_pbr_bundle(cursor_model, false);
        cursor_bundle.transform.translation.y = 0.1;
        cursor_bundle.transform.scale = Vec3::new(1.1, 1.0, 1.1);

        self.commands
            .entity(player_entity)
            .insert(Player)
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
            )
            .unwrap();
        self.commands
            .entity(player)
            .insert(TextLabel::new(self.sav.player.name.clone()));
        self.configure_player(player);

        let survivor = self
            .spawn_character(
                parent,
                Pos::new(10, Level::ZERO, 10).offset(offset).unwrap(),
                ObjectId::new("human"),
            )
            .unwrap();
        self.commands
            .entity(survivor)
            .insert(TextLabel::new("Survivor"));

        self.spawn_character(
            parent,
            Pos::new(12, Level::ZERO, 16).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.spawn_character(
            parent,
            Pos::new(40, Level::ZERO, 40).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.spawn_character(
            parent,
            Pos::new(38, Level::ZERO, 39).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.spawn_character(
            parent,
            Pos::new(37, Level::ZERO, 37).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.spawn_character(
            parent,
            Pos::new(34, Level::ZERO, 34).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
    }
}
