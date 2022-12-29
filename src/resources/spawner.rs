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
    pub(crate) zone_level_ids: ResMut<'w, ZoneLevelIds>,
    pub(crate) explored: ResMut<'w, Explored>,
    infos: ResMut<'w, Infos>,
    paths: Res<'w, Paths>,
    sav: Res<'w, Sav>,
}

impl<'w, 's> TileSpawner<'w, 's> {
    fn get_tile_mesh(&mut self, model: &Model) -> Handle<Mesh> {
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
            mesh: self.get_tile_mesh(model),
            material: if shaded {
                Handle::<StandardMaterial>::default()
            } else {
                self.get_appearance(model).material(&LastSeen::Currently)
            },
            transform: model.to_transform(),
            visibility: if shaded {
                Visibility::default()
            } else {
                Visibility::INVISIBLE
            },
            ..PbrBundle::default()
        }
    }

    fn spawn_character(&mut self, parent: Entity, pos: Pos, id: &ObjectId) {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Character,
            id,
        };
        let entity = self.spawn_tile(parent, pos, definition);
        self.commands
            .entity(entity)
            .insert(Obstacle)
            .insert(Health::new(5))
            .insert(Faction::Animal)
            .insert(BaseSpeed::from_h_kmph(10))
            .insert(Container(0));

        let character_info = self
            .infos
            .character(definition.id)
            .unwrap_or_else(|| panic!("{:?}", definition.id));
        if character_info.flags.aquatic() {
            self.commands.entity(entity).insert(Aquatic);
        }
    }

    fn spawn_item(&mut self, parent: Entity, pos: Pos, id: &ObjectId, amount: Amount) {
        let definition = &ObjectDefinition {
            category: ObjectCategory::Item,
            id,
        };
        let entity = self.spawn_tile(parent, pos, definition);
        self.commands.entity(entity).insert(amount);
    }

    fn spawn_tile(&mut self, parent: Entity, pos: Pos, definition: &ObjectDefinition) -> Entity {
        let models = self
            .loader
            .get_models(definition, &self.infos.variants(definition));
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

        let label = self.infos.label(definition, 1);

        let tile = self
            .commands
            .spawn(SpatialBundle::default())
            .insert(Visibility::INVISIBLE)
            .insert(label)
            .insert(pos)
            .insert(Transform::from_translation(pos.vec3()))
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

        self.commands.entity(parent).add_child(tile);

        if definition.category.shading_applied() {
            self.commands.entity(tile).insert(last_seen);
        }

        match definition.category {
            ObjectCategory::Terrain => {
                let terrain_info = self
                    .infos
                    .terrain(definition.id)
                    .unwrap_or_else(|| panic!("{:?}", definition.id));
                match terrain_info {
                    TerrainInfo::Terrain {
                        move_cost, flags, ..
                    } => {
                        let move_cost = *move_cost;
                        if 0 < move_cost.0 {
                            self.commands.entity(tile).insert(Floor {
                                water: flags.water(),
                                move_cost,
                            });
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
                        }
                    }
                    TerrainInfo::FieldType { .. } => {}
                }
            }
            ObjectCategory::Furniture => {
                let furniture_info = self
                    .infos
                    .furniture(definition.id)
                    .unwrap_or_else(|| panic!("{:?}", definition.id));

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
            _ => {
                // pass
            }
        };

        tile
    }

    pub(crate) fn spawn_expanded_subzone_level(
        &mut self,
        subzone_level: SubzoneLevel,
    ) -> Result<(), serde_json::Error> {
        let map_path = MapPath::new(&self.paths.world_path(), ZoneLevel::from(subzone_level));
        if let Some(submap) = Option::<Map>::try_from(map_path)?
            .map(|map| map.0.into_iter().nth(subzone_level.index()).unwrap())
            .or_else(|| Submap::fallback(subzone_level))
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
                let id = terrain.get(&pos).unwrap();
                if id != &&ObjectId::new("t_open_air") && id != &&ObjectId::new("t_open_air_rooved")
                {
                    self.spawn_tile(
                        subzone_level_entity,
                        pos,
                        &ObjectDefinition {
                            category: ObjectCategory::Terrain,
                            id,
                        },
                    );
                }

                for id in submap
                    .furniture
                    .iter()
                    .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                {
                    self.spawn_tile(
                        subzone_level_entity,
                        pos,
                        &ObjectDefinition {
                            category: ObjectCategory::Furniture,
                            id,
                        },
                    );
                }

                for repetitions in submap
                    .items
                    .0
                    .iter()
                    .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                {
                    for repetition in repetitions {
                        let CddaAmount { obj: item, amount } = repetition.as_amount();
                        self.spawn_item(
                            subzone_level_entity,
                            pos,
                            id,
                            Amount(item.charges.unwrap_or(1) * amount),
                        );
                    }
                }

                for spawn in submap
                    .spawns
                    .iter()
                    .filter(|spawn| spawn.x == x && spawn.z == z)
                {
                    self.spawn_character(subzone_level_entity, pos, &spawn.id);
                }

                for fields in submap
                    .fields
                    .0
                    .iter()
                    .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                {
                    for field in &fields.0 {
                        self.spawn_tile(
                            subzone_level_entity,
                            pos,
                            &ObjectDefinition {
                                category: ObjectCategory::Terrain,
                                id: &field.id,
                            },
                        );
                    }
                }
            }
        }
    }

    pub(crate) fn spawn_zones_around(&mut self, center: Zone) {
        let distance = 100;
        for x in -distance..=distance {
            for z in -distance..=distance {
                let zone = center.offset(x, z);
                for level in Level::ALL {
                    let zone_level = zone.zone_level(level);
                    let id = self
                        .zone_level_ids
                        .get(zone_level)
                        .map(std::clone::Clone::clone);
                    if let Some(id) = id {
                        self.spawn_collaped_zone_level(
                            zone_level,
                            &ObjectDefinition {
                                category: ObjectCategory::ZoneLevel,
                                id: &id,
                            },
                        );
                    }
                }
            }
        }
    }

    fn spawn_collaped_zone_level(&mut self, zone_level: ZoneLevel, definition: &ObjectDefinition) {
        let pbr_bundles = if definition.id.is_hidden_zone() {
            Vec::new()
        } else {
            //println!("zone_level: {zone_level:?} {:?}", &definition);
            self.loader
                .get_models(definition, &self.infos.variants(definition))
                .iter()
                .map(|model| self.get_pbr_bundle(model, false))
                .collect::<Vec<PbrBundle>>()
        };

        let label = self.infos.label(definition, 1);

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
                        (LastSeen::Never, Visibility::INVISIBLE)
                    } else {
                        (LastSeen::Previously, Visibility::VISIBLE)
                    },
                )
                .with_children(|child_builder| {
                    for pbr_bundle in pbr_bundles {
                        child_builder.spawn(pbr_bundle);
                    }
                });
        }
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
        self.tile_spawner.spawn_tile(
            parent,
            pos,
            &ObjectDefinition {
                category: ObjectCategory::Terrain,
                id: &ObjectId::new("t_wood_stairs_down"),
            },
        );
    }

    pub(crate) fn spawn_roofing(&mut self, parent: Entity, pos: Pos) {
        self.tile_spawner.spawn_tile(
            parent,
            pos,
            &ObjectDefinition {
                category: ObjectCategory::Terrain,
                id: &ObjectId::new("t_shingle_flat_roof"),
            },
        );
    }

    pub(crate) fn spawn_wall(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(Wall)
            .insert(pos)
            .insert(Integrity::new(1000))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(Label::new("wall"))
            .insert(LastSeen::Never)
            .insert(Visibility::INVISIBLE)
            .with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.wall_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wooden_wall.clone());
            });
    }

    pub(crate) fn spawn_window(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(Window)
            .insert(pos)
            .insert(Integrity::new(1))
            .insert(Obstacle)
            .insert(Hurdle(MoveCostMod(2)))
            .insert(Label::new("window"))
            .insert(LastSeen::Never)
            .insert(Visibility::INVISIBLE)
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
                    .insert(self.custom.glass.clone())
                    .insert(WindowPane);
            });
    }

    pub(crate) fn spawn_stairs(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(StairsUp)
            .insert(pos)
            .insert(Integrity::new(100))
            .insert(Hurdle(MoveCostMod(1)))
            .insert(Label::new("stairs"))
            .insert(LastSeen::Never)
            .insert(Visibility::INVISIBLE)
            .with_children(|child_builder| {
                child_builder
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.stair_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wood.clone());
            });
    }

    pub(crate) fn spawn_rack(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(Rack)
            .insert(pos)
            .insert(Integrity::new(40))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(Label::new("rack"))
            .insert(LastSeen::Never)
            .insert(Visibility::INVISIBLE)
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
        self.tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(Table)
            .insert(pos)
            .insert(Integrity::new(30))
            .insert(Hurdle(MoveCostMod(2)))
            .insert(Label::new("table"))
            .insert(LastSeen::Never)
            .insert(Visibility::INVISIBLE)
            .with_children(|parent| {
                parent
                    .spawn(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.table_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wood.clone());
            });
    }

    pub(crate) fn spawn_chair(&mut self, pos: Pos) {
        let scale = 0.45 * Millimeter::ADJACENT.f32();

        self.tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(Chair)
            .insert(pos)
            .insert(Integrity::new(10))
            .insert(Hurdle(MoveCostMod(3)))
            .insert(Label::new("chair"))
            .insert(LastSeen::Never)
            .insert(Visibility::INVISIBLE)
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
            });
    }

    fn configure_player(&mut self, player_entity: Entity, player: Player) {
        let cursor_definition = ObjectDefinition {
            category: ObjectCategory::Meta,
            id: &ObjectId::new("cursor"),
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
            .insert(player)
            .with_children(|child_builder| {
                child_builder
                    .spawn(SpatialBundle::default())
                    .insert(CameraBase)
                    .with_children(|child_builder| {
                        child_builder.spawn(cursor_bundle).insert(ExamineCursor);

                        let camera_direction = Transform::IDENTITY
                            .looking_at(Vec3::new(1.0, 0.0, 0.1), Vec3::Y)
                            * Transform::from_translation(Vec3::new(0.0, 1.0, 0.0));
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

    pub(crate) fn spawn_character(
        &mut self,
        label: Label,
        pos: Pos,
        health: Health,
        speed: BaseSpeed,
        faction: Faction,
        player: Option<Player>,
    ) {
        let character_scale = 1.5;
        let character_definition = ObjectDefinition {
            category: ObjectCategory::Character,
            id: &match faction {
                Faction::Human => ObjectId::new("overlay_male_mutation_SKIN_TAN"),
                _ => ObjectId::new("mon_zombie"),
            },
        };
        let character_model = &mut self.tile_spawner.loader.get_models(
            &character_definition,
            &vec![character_definition.id.clone()],
        )[0];
        if let ModelShape::Plane {
            ref mut transform2d,
            ..
        } = character_model.shape
        {
            transform2d.scale *= character_scale;
        }
        let mut character_bundle = self.tile_spawner.get_pbr_bundle(character_model, false);
        character_bundle.transform.translation.y *= character_scale;
        println!("{:?}", character_bundle.transform);

        let character_appearance = self.tile_spawner.get_appearance(character_model);

        let entity = self
            .tile_spawner
            .commands
            .spawn(SpatialBundle::default())
            .insert(label)
            .insert(pos)
            .insert(LastSeen::Never)
            .insert(health)
            .insert(speed)
            .insert(faction)
            .insert(Obstacle)
            .insert(Container(4))
            .with_children(|child_builder| {
                child_builder
                    .spawn(character_bundle)
                    .insert(Visibility::VISIBLE)
                    .insert(character_appearance);
            })
            .id();

        if let Some(player) = player {
            self.configure_player(entity, player);
        }
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
                shadow_projection: OrthographicProjection {
                    left: -0.35,
                    right: 500.35,
                    bottom: -0.1,
                    top: 5.0,
                    near: -5.0,
                    far: 5.0,
                    ..OrthographicProjection::default()
                },
                //shadows_enabled: true, // TODO transparency should not be ignored
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
        self.spawn_character(
            Label::new(self.tile_spawner.sav.player.name.clone()),
            Pos::new(45, Level::ZERO, 56)
                .offset(offset)
                .unwrap()
                .offset(PosOffset {
                    x: -9,
                    level: LevelOffset::ZERO,
                    z: 0,
                })
                .unwrap(),
            Health::new(10),
            BaseSpeed::from_h_kmph(6),
            Faction::Human,
            Some(Player {
                state: PlayerActionState::Normal,
                camera_distance: 7.1,
            }),
        );
        self.spawn_character(
            Label::new("Survivor"),
            Pos::new(10, Level::ZERO, 10).offset(offset).unwrap(),
            Health::new(3),
            BaseSpeed::from_h_kmph(7),
            Faction::Human,
            None,
        );
        self.spawn_character(
            Label::new("Zombie one"),
            Pos::new(12, Level::ZERO, 16).offset(offset).unwrap(),
            Health::new(3),
            BaseSpeed::from_h_kmph(5),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie two"),
            Pos::new(40, Level::ZERO, 40).offset(offset).unwrap(),
            Health::new(3),
            BaseSpeed::from_h_kmph(7),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie three"),
            Pos::new(38, Level::ZERO, 39).offset(offset).unwrap(),
            Health::new(3),
            BaseSpeed::from_h_kmph(8),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie four"),
            Pos::new(37, Level::ZERO, 37).offset(offset).unwrap(),
            Health::new(3),
            BaseSpeed::from_h_kmph(9),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie five"),
            Pos::new(34, Level::ZERO, 34).offset(offset).unwrap(),
            Health::new(3),
            BaseSpeed::from_h_kmph(3),
            Faction::Zombie,
            None,
        );
    }
}
