use crate::prelude::*;
use bevy::{
    ecs::system::{Insert, SystemParam},
    math::Quat,
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

#[derive(SystemParam)]
pub(crate) struct TileSpawner<'w, 's> {
    commands: Commands<'w, 's>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    asset_server: Res<'w, AssetServer>,
    loader: Res<'w, TileLoader>,
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

impl<'w, 's> TileSpawner<'w, 's> {
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

    fn spawn_item(&mut self, parent: Entity, pos: Pos, item: &CddaItem, amount: Amount) {
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
            println!("No info found for {:?}. Spawning skipped", definition.id);
            return;
        };

        let (volume, mass) = match &item.corpse {
            Some(corpse_id) if corpse_id != &ObjectId::new("mon_null") => {
                println!("{:?}", &corpse_id);
                let monster_info = self.infos.character(corpse_id).unwrap();
                (monster_info.volume, monster_info.mass)
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
        subzone_level: SubzoneLevel,
    ) -> Result<(), serde_json::Error> {
        let map_path = MapPath::new(&self.paths.world_path(), ZoneLevel::from(subzone_level));
        if let Some(submap) = Option::<Map>::try_from(map_path)?
            .map(|map| map.0.into_iter().nth(subzone_level.index()).unwrap())
            .or_else(|| {
                let object_id = self.zone_level_ids.get(ZoneLevel::from(subzone_level));
                Submap::fallback(subzone_level, object_id)
            })
        {
            assert_eq!(
                submap.coordinates,
                subzone_level.coordinates(),
                "{:?} {:?}",
                submap.coordinates,
                subzone_level.coordinates()
            );
            self.spawn_subzone(&submap, subzone_level);
        }
        Ok(())
    }

    fn spawn_subzone(&mut self, submap: &Submap, subzone_level: SubzoneLevel) {
        //println!("{:?} started...", subzone_level);
        let subzone_level_entity = self
            .commands
            .spawn(SpatialBundle::default())
            .insert(subzone_level)
            .id();

        let base_pos = subzone_level.base_pos();
        let terrain = submap.terrain.load_as_subzone(subzone_level);

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
                if id != &ObjectId::new("t_open_air") && id != &ObjectId::new("t_open_air_rooved") {
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
                        self.spawn_item(
                            subzone_level_entity,
                            pos,
                            item,
                            Amount(item.charges.unwrap_or(1) * amount),
                        );
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

        self.subzone_level_entities
            .add(subzone_level, subzone_level_entity);
    }

    pub(crate) fn spawn_collapsed_zone_level(
        &mut self,
        zone_level: ZoneLevel,
        child_visibiltiy: &Visibility,
    ) {
        assert!(zone_level.level <= Level::ZERO);

        let definition = ObjectDefinition {
            category: ObjectCategory::ZoneLevel,
            id: self.zone_level_ids.get(zone_level).clone(),
        };

        let pbr_bundles = if definition.id.is_hidden_zone() {
            Vec::new()
        } else {
            //println!("zone_level: {zone_level:?} {:?}", &definition);
            self.loader
                .get_models(&definition, &self.infos.variants(&definition))
                .iter()
                .map(|model| self.get_pbr_bundle(model, false))
                .collect::<Vec<PbrBundle>>()
        };

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
                .insert(
                    if self.explored.has_zone_level_been_seen(zone_level) == SeenFrom::Never {
                        (LastSeen::Never, Visibility::Hidden)
                    } else {
                        (LastSeen::Previously, Visibility::Inherited)
                    },
                )
                .with_children(|child_builder| {
                    for pbr_bundle in pbr_bundles {
                        child_builder.spawn(pbr_bundle).insert(*child_visibiltiy);
                    }
                });
        }

        self.zone_level_entities.add(zone_level, entity.id());
    }
}

#[derive(Resource)]
pub(crate) struct CustomData {
    glass: Appearance,
    wood: Appearance,
    whitish: Appearance,
    wooden_wall: Appearance,
    cube_mesh: Handle<Mesh>,
    wall_transform: Transform,
    window_pane_transform: Transform,
    stair_transform: Transform,
    rack_transform: Transform,
    table_transform: Transform,
}

impl CustomData {
    pub(crate) fn new(
        material_assets: &mut Assets<StandardMaterial>,
        mesh_assets: &mut Assets<Mesh>,
        asset_server: &AssetServer,
    ) -> Self {
        Self {
            glass: Appearance::new(material_assets, Color::rgba(0.8, 0.9, 1.0, 0.2)), // transparant blue
            wood: Appearance::new(material_assets, Color::rgb(0.7, 0.6, 0.5)),
            whitish: Appearance::new(material_assets, Color::rgb(0.95, 0.93, 0.88)),
            wooden_wall: Appearance::new(
                material_assets,
                asset_server.load(Paths::tiles_path().join("wall.png")),
            ),
            cube_mesh: mesh_assets.add(Mesh::from(shape::Cube { size: 1.0 })),
            wall_transform: Transform {
                translation: Vec3::new(0.0, 0.495 * Millimeter::VERTICAL.f32(), 0.0),
                scale: Vec3::new(
                    Millimeter::ADJACENT.f32(),
                    0.99 * Millimeter::VERTICAL.f32(),
                    Millimeter::ADJACENT.f32(),
                ),
                ..Transform::default()
            },
            window_pane_transform: Transform {
                translation: Vec3::new(0.0, 0.75, 0.0),
                rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                scale: Vec3::new(
                    0.99 * Millimeter::ADJACENT.f32(),
                    0.99 * Millimeter::VERTICAL.f32(),
                    0.99 * Millimeter::ADJACENT.f32(),
                ),
            },
            stair_transform: Transform {
                rotation: Quat::from_rotation_x(-0.12 * std::f32::consts::PI),
                scale: Vec3::new(0.8, 1.2 * Millimeter::VERTICAL.f32(), 0.2),
                ..Transform::default()
            },
            rack_transform: Transform {
                translation: Vec3::new(0.0, 0.45 * Millimeter::VERTICAL.f32(), 0.0),
                scale: Vec3::new(
                    0.90 * Millimeter::ADJACENT.f32(),
                    0.90 * Millimeter::VERTICAL.f32(),
                    0.90 * Millimeter::ADJACENT.f32(),
                ),
                ..default()
            },
            table_transform: Transform {
                translation: Vec3::new(0.0, 0.375 * Millimeter::ADJACENT.f32(), 0.0),
                scale: Vec3::new(
                    Millimeter::ADJACENT.f32(),
                    0.75 * Millimeter::ADJACENT.f32(),
                    Millimeter::ADJACENT.f32(),
                ),
                ..Transform::default()
            },
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct Spawner<'w, 's> {
    tile_spawner: TileSpawner<'w, 's>,
    custom: ResMut<'w, CustomData>,
}

impl<'w, 's> Spawner<'w, 's> {
    pub(crate) fn spawn_stairs_down(&mut self, parent: Entity, pos: Pos) {
        self.tile_spawner
            .spawn_terrain(parent, pos, ObjectId::new("t_wood_stairs_down"));
    }

    pub(crate) fn spawn_roofing(&mut self, parent: Entity, pos: Pos) {
        self.tile_spawner
            .spawn_terrain(parent, pos, ObjectId::new("t_shingle_flat_roof"));
    }

    pub(crate) fn spawn_wall(&mut self, pos: Pos) {
        let tile = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(pos)
            .insert(Integrity::new(1000))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(OpaqueFloor)
            .insert(TextLabel::new("wall"))
            .insert(LastSeen::Never)
            .insert(Visibility::Hidden)
            .with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.wall_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wooden_wall.clone());
            })
            .id();

        self.tile_spawner.location.update(tile, Some(pos));
    }

    pub(crate) fn spawn_window(&mut self, pos: Pos) {
        let tile = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(pos)
            .insert(Integrity::new(1))
            .insert(Obstacle)
            .insert(Hurdle(MoveCostMod(2)))
            .insert(OpaqueFloor)
            .insert(TextLabel::new("window"))
            .insert(LastSeen::Never)
            .insert(Visibility::Hidden)
            .with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wooden_wall.clone());

                parent
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.window_pane_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.glass.clone());
            })
            .id();

        self.tile_spawner.location.update(tile, Some(pos));
    }

    pub(crate) fn spawn_stairs(&mut self, pos: Pos) {
        let tile = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(StairsUp)
            .insert(pos)
            .insert(Integrity::new(100))
            .insert(Hurdle(MoveCostMod(1)))
            .insert(TextLabel::new("stairs"))
            .insert(LastSeen::Never)
            .insert(Visibility::Hidden)
            .with_children(|child_builder| {
                child_builder
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.stair_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wood.clone());
            })
            .id();

        self.tile_spawner.location.update(tile, Some(pos));
    }

    pub(crate) fn spawn_rack(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(pos)
            .insert(Integrity::new(40))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(TextLabel::new("rack"))
            .insert(LastSeen::Never)
            .insert(Visibility::Hidden)
            .with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.rack_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wood.clone());
            });
    }

    pub(crate) fn spawn_table(&mut self, pos: Pos) {
        let tile = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(pos)
            .insert(Integrity::new(30))
            .insert(Hurdle(MoveCostMod(2)))
            .insert(TextLabel::new("table"))
            .insert(LastSeen::Never)
            .insert(Visibility::Hidden)
            .with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.table_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wood.clone());
            })
            .id();

        self.tile_spawner.location.update(tile, Some(pos));
    }

    pub(crate) fn spawn_chair(&mut self, pos: Pos) {
        let scale = 0.45 * Millimeter::ADJACENT.f32();

        let tile = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(pos)
            .insert(Integrity::new(10))
            .insert(Hurdle(MoveCostMod(3)))
            .insert(TextLabel::new("chair"))
            .insert(LastSeen::Never)
            .insert(Visibility::Hidden)
            .with_children(|child_builder| {
                child_builder
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: Transform {
                            translation: Vec3::new(0.0, scale / 2.0, 0.0),
                            scale: Vec3::splat(scale),
                            ..Transform::default()
                        },
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.whitish.clone());

                child_builder
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: Transform {
                            translation: Vec3::new(0.0, 0.425, -0.23),
                            scale: Vec3::new(scale, 0.85, 0.05),
                            ..Transform::default()
                        },
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.whitish.clone());
            })
            .id();

        self.tile_spawner.location.update(tile, Some(pos));
    }

    fn configure_player(&mut self, player_entity: Entity) {
        let cursor_definition = ObjectDefinition {
            category: ObjectCategory::Meta,
            id: ObjectId::new("cursor"),
        };
        let cursor_model = &mut self
            .tile_spawner
            .loader
            .get_models(&cursor_definition, &vec![cursor_definition.id.clone()])[0];
        let mut cursor_bundle = self.tile_spawner.get_pbr_bundle(cursor_model, false);
        cursor_bundle.transform.translation.y = 0.1;
        cursor_bundle.transform.scale = Vec3::new(1.1, 1.0, 1.1);

        self.tile_spawner
            .commands
            .entity(player_entity)
            .insert(Player {
                state: PlayerActionState::Normal,
                camera_distance: 7.1,
            })
            .with_children(|child_builder| {
                child_builder
                    .spawn(SpatialBundle::default())
                    .insert(CameraBase)
                    .with_children(|child_builder| {
                        child_builder.spawn(cursor_bundle).insert(ExamineCursor);

                        let camera_direction = Transform::IDENTITY
                            .looking_at(Vec3::new(1.0, 0.0, 0.1), Vec3::Y)
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
        self.tile_spawner.commands.spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 50_000.0,
                shadows_enabled: false, // TODO shadow direction does not match buildin shadows
                ..DirectionalLight::default()
            },
            transform: light_transform,
            ..DirectionalLightBundle::default()
        });
    }

    pub(crate) fn spawn_floors(&mut self, offset: PosOffset) {
        let parent = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .id();

        for y in 1..=2 {
            for x in 17..24 {
                let z_range = match y {
                    1 => 17..26,
                    _ => 21..26,
                };
                for z in z_range {
                    let pos = Pos::new(x, Level::new(y), z);
                    match pos {
                        Pos {
                            x: 18,
                            level: Level { h: 1 },
                            z: 24,
                        }
                        | Pos {
                            x: 18,
                            level: Level { h: 2 },
                            z: 22,
                        } => {
                            self.spawn_stairs_down(parent, pos.offset(offset).unwrap());
                        }
                        _ => {
                            self.spawn_roofing(parent, pos.offset(offset).unwrap());
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn spawn_house(&mut self, offset: PosOffset) {
        self.spawn_wall(Pos::new(17, Level::ZERO, 17).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::ZERO, 18).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::ZERO, 19).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::ZERO, 20).offset(offset).unwrap());
        // z 21: door
        self.spawn_wall(Pos::new(17, Level::ZERO, 22).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::ZERO, 23).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::ZERO, 24).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::ZERO, 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(18, Level::ZERO, 25).offset(offset).unwrap());
        self.spawn_window(Pos::new(19, Level::ZERO, 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(20, Level::ZERO, 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(21, Level::ZERO, 25).offset(offset).unwrap());
        // x 22: door
        self.spawn_wall(Pos::new(23, Level::ZERO, 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::ZERO, 24).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::ZERO, 23).offset(offset).unwrap());
        self.spawn_window(Pos::new(23, Level::ZERO, 22).offset(offset).unwrap());
        self.spawn_window(Pos::new(23, Level::ZERO, 21).offset(offset).unwrap());
        self.spawn_window(Pos::new(23, Level::ZERO, 20).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::ZERO, 19).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::ZERO, 18).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::ZERO, 17).offset(offset).unwrap());
        self.spawn_wall(Pos::new(22, Level::ZERO, 17).offset(offset).unwrap());
        self.spawn_wall(Pos::new(21, Level::ZERO, 17).offset(offset).unwrap());
        self.spawn_window(Pos::new(20, Level::ZERO, 17).offset(offset).unwrap());
        self.spawn_wall(Pos::new(19, Level::ZERO, 17).offset(offset).unwrap());
        self.spawn_wall(Pos::new(18, Level::ZERO, 17).offset(offset).unwrap());

        self.spawn_stairs(Pos::new(18, Level::ZERO, 24).offset(offset).unwrap());
        self.spawn_rack(Pos::new(18, Level::ZERO, 18).offset(offset).unwrap());
        self.spawn_rack(Pos::new(20, Level::ZERO, 24).offset(offset).unwrap());
        self.spawn_chair(Pos::new(19, Level::ZERO, 20).offset(offset).unwrap());
        self.spawn_table(Pos::new(19, Level::ZERO, 21).offset(offset).unwrap());
        self.spawn_chair(Pos::new(19, Level::ZERO, 22).offset(offset).unwrap());
        self.spawn_chair(Pos::new(20, Level::ZERO, 20).offset(offset).unwrap());
        self.spawn_table(Pos::new(20, Level::ZERO, 21).offset(offset).unwrap());
        self.spawn_chair(Pos::new(20, Level::ZERO, 22).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::ZERO, 22).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::ZERO, 21).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::ZERO, 20).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::ZERO, 19).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::ZERO, 18).offset(offset).unwrap());
        self.spawn_table(Pos::new(21, Level::ZERO, 18).offset(offset).unwrap());
        self.spawn_table(Pos::new(20, Level::ZERO, 18).offset(offset).unwrap());

        self.spawn_wall(Pos::new(17, Level::new(1), 21).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::new(1), 22).offset(offset).unwrap());
        self.spawn_window(Pos::new(17, Level::new(1), 23).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::new(1), 24).offset(offset).unwrap());
        self.spawn_wall(Pos::new(17, Level::new(1), 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(18, Level::new(1), 25).offset(offset).unwrap());
        self.spawn_window(Pos::new(19, Level::new(1), 25).offset(offset).unwrap());
        self.spawn_window(Pos::new(20, Level::new(1), 25).offset(offset).unwrap());
        self.spawn_window(Pos::new(21, Level::new(1), 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(22, Level::new(1), 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::new(1), 25).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::new(1), 24).offset(offset).unwrap());
        self.spawn_window(Pos::new(23, Level::new(1), 23).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::new(1), 22).offset(offset).unwrap());
        self.spawn_wall(Pos::new(23, Level::new(1), 21).offset(offset).unwrap());
        self.spawn_wall(Pos::new(22, Level::new(1), 21).offset(offset).unwrap());
        self.spawn_wall(Pos::new(21, Level::new(1), 21).offset(offset).unwrap());
        // x 20: door
        self.spawn_wall(Pos::new(19, Level::new(1), 21).offset(offset).unwrap());
        self.spawn_wall(Pos::new(18, Level::new(1), 21).offset(offset).unwrap());

        self.spawn_stairs(Pos::new(18, Level::new(1), 22).offset(offset).unwrap());
        self.spawn_table(Pos::new(21, Level::new(1), 24).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::new(1), 24).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::new(1), 23).offset(offset).unwrap());
        self.spawn_table(Pos::new(22, Level::new(1), 22).offset(offset).unwrap());
        self.spawn_chair(Pos::new(21, Level::new(1), 23).offset(offset).unwrap());
    }

    pub(crate) fn spawn_characters(&mut self, offset: PosOffset) {
        let custom_character_parent = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .id();

        let player = self
            .tile_spawner
            .spawn_character(
                custom_character_parent,
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
        self.tile_spawner
            .commands
            .entity(player)
            .insert(TextLabel::new(self.tile_spawner.sav.player.name.clone()));
        self.configure_player(player);

        let survivor = self
            .tile_spawner
            .spawn_character(
                custom_character_parent,
                Pos::new(10, Level::ZERO, 10).offset(offset).unwrap(),
                ObjectId::new("human"),
            )
            .unwrap();
        self.tile_spawner
            .commands
            .entity(survivor)
            .insert(TextLabel::new("Survivor"));

        self.tile_spawner.spawn_character(
            custom_character_parent,
            Pos::new(12, Level::ZERO, 16).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.tile_spawner.spawn_character(
            custom_character_parent,
            Pos::new(40, Level::ZERO, 40).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.tile_spawner.spawn_character(
            custom_character_parent,
            Pos::new(38, Level::ZERO, 39).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.tile_spawner.spawn_character(
            custom_character_parent,
            Pos::new(37, Level::ZERO, 37).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
        self.tile_spawner.spawn_character(
            custom_character_parent,
            Pos::new(34, Level::ZERO, 34).offset(offset).unwrap(),
            ObjectId::new("mon_zombie"),
        );
    }
}
