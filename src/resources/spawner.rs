use crate::prelude::*;
use bevy::ecs::system::{Insert, Remove, SystemParam};
use bevy::math::Quat;
use bevy::prelude::*;
use bevy::render::camera::{PerspectiveProjection, Projection::Perspective};
use bevy::utils::HashMap;

#[derive(Default)]
pub(crate) struct TileCaches {
    appearance_cache: HashMap<String, Appearance>,
    plane_mesh_cache: HashMap<SpriteNumber, Handle<Mesh>>,
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
    zone_level_names: ResMut<'w, ZoneLevelNames>,
    explored: ResMut<'w, Explored>,
    item_infos: ResMut<'w, ItemInfos>,
    paths: Res<'w, Paths>,
    sav: Res<'w, Sav>,
}

impl<'w, 's> TileSpawner<'w, 's> {
    fn get_tile_mesh(&mut self, model: &Model) -> Handle<Mesh> {
        match model.shape {
            ModelShape::Plane { .. } => &mut self.caches.plane_mesh_cache,
            ModelShape::Cuboid { .. } => &mut self.caches.cuboid_mesh_cache,
        }
        .entry(model.sprite_number)
        .or_insert_with(|| self.mesh_assets.add(model.to_mesh()))
        .clone()
    }

    fn get_appearance(&mut self, model: &Model) -> Appearance {
        self.caches
            .appearance_cache
            .entry(model.texture_path.to_string())
            .or_insert_with(|| {
                let material = StandardMaterial {
                    base_color_texture: Some(self.asset_server.load(&model.texture_path)),
                    alpha_mode: model.alpha_mode,
                    ..StandardMaterial::default()
                };
                Appearance::new(&mut self.material_assets, material)
            })
            .clone()
    }

    fn get_pbr_bundle(&mut self, model: &Model) -> PbrBundle {
        PbrBundle {
            mesh: self.get_tile_mesh(model),
            material: self.get_appearance(model).material(&LastSeen::Currently), // required when not shaded
            transform: model.to_transform(),
            ..PbrBundle::default()
        }
    }

    fn spawn_tile(&mut self, parent: Entity, pos: Pos, definition: ObjectDefinition) {
        let models = self.loader.get_models(&definition);
        let child_info = models
            .iter()
            .map(|model| (self.get_pbr_bundle(model), self.get_appearance(model)))
            .collect::<Vec<(PbrBundle, Appearance)>>();

        let item_info = self.item_infos.get(definition.name);

        self.commands.entity(parent).with_children(|child_builder| {
            let tile = child_builder
                .spawn_bundle(SpatialBundle::default())
                .insert(
                    item_info
                        .map_or_else(|| definition.name.to_fallback_label(), |i| i.to_label(1)),
                )
                .insert(pos)
                .insert(Transform::from_translation(pos.vec3()))
                .with_children(|child_builder| {
                    for (pbr_bundle, apprearance) in child_info {
                        child_builder.spawn_bundle(pbr_bundle).insert(apprearance);
                    }
                })
                .id();

            if definition.specifier.shading_applied() {
                child_builder.add_command(Insert {
                    entity: tile,
                    component: if self.explored.has_been_seen(ZoneLevel::from(pos)) {
                        LastSeen::Previously
                    } else {
                        LastSeen::Never
                    },
                });
            }

            match definition.specifier {
                ObjectSpecifier::Character => {
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: Obstacle,
                    });
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: Health::new(5),
                    });
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: Faction::Animal,
                    });
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: Speed::from_h_kmph(10),
                    });
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: Container(0),
                    });
                }
                ObjectSpecifier::Item(item) => {
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: item,
                    });
                }
                _ => {
                    if definition.specifier == ObjectSpecifier::Terrain {
                        child_builder.add_command(Insert {
                            entity: tile,
                            component: Floor,
                        });
                    }

                    let up = definition.name.is_stairs_up();
                    let down = definition.name.is_stairs_down();
                    if up || down {
                        // can be both

                        if up {
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: StairsUp,
                            });
                        }

                        if down {
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: StairsDown,
                            });
                        }
                    } else {
                        TileSpawner::add_components_from_shape(
                            models
                                .iter()
                                .find(|m| m.layer == SpriteLayer::Front)
                                .unwrap()
                                .shape,
                            tile,
                            child_builder,
                        );
                    }
                }
            }
        });
    }

    fn add_components_from_shape(
        shape: ModelShape,
        tile: Entity,
        child_builder: &mut ChildBuilder,
    ) {
        match shape {
            ModelShape::Plane {
                orientation,
                transform2d,
            } => {
                if orientation == SpriteOrientation::Vertical {
                    match transform2d.scale.y {
                        y if 2.5 < y => {
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: Obstacle,
                            });
                            child_builder.add_command(Remove {
                                entity: tile,
                                phantom: core::marker::PhantomData::<Floor>,
                            });
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: Opaque,
                            });
                        }
                        y if 2.0 < y => {
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: Hurdle(2.0),
                            });
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: Opaque,
                            });
                        }
                        _ => {
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: Hurdle(1.5),
                            });
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: Integrity::new(10),
                            });
                        }
                    }
                }
            }
            ModelShape::Cuboid { .. } => {
                child_builder.add_command(Insert {
                    entity: tile,
                    component: Obstacle,
                });
                child_builder.add_command(Insert {
                    entity: tile,
                    component: Opaque,
                });
            }
        }
    }

    pub(crate) fn spawn_expanded_zone_level(
        &mut self,
        zone_level: ZoneLevel,
    ) -> Result<(), serde_json::Error> {
        let map_path = MapPath::new(&self.paths.world_path(), zone_level);
        if let Some(map) = Option::<Map>::try_from(map_path)? {
            let zone_level_entity = self
                .commands
                .spawn_bundle(SpatialBundle::default())
                .insert(zone_level)
                .id();
            let base = map.0[0].coordinates;
            for submap in &map.0 {
                assert!(base.2 == submap.coordinates.2);
                self.spawn_subzone(
                    submap,
                    zone_level_entity,
                    zone_level,
                    Pos::new(
                        12 * (submap.coordinates.0 - base.0),
                        Level::ZERO,
                        12 * (submap.coordinates.1 - base.1),
                    ),
                );
            }
        }
        Ok(())
    }

    fn spawn_subzone(
        &mut self,
        submap: &Submap,
        parent_entity: Entity,
        zone_level: ZoneLevel,
        subzone_offset: Pos,
    ) {
        let base_pos = zone_level.base_pos().offset(subzone_offset).unwrap();
        let terrain = submap.terrain.load_as_subzone(base_pos);

        for x in 0..12 {
            for z in 0..12 {
                let pos = base_pos.offset(Pos::new(x, Level::ZERO, z)).unwrap();
                let tile_name = terrain.get(&pos).unwrap();
                if tile_name != &&ObjectName::new("t_open_air")
                    && tile_name != &&ObjectName::new("t_open_air_rooved")
                {
                    self.spawn_tile(
                        parent_entity,
                        pos,
                        ObjectDefinition {
                            name: tile_name,
                            specifier: ObjectSpecifier::Terrain,
                        },
                    );
                }

                for tile_name in submap
                    .furniture
                    .iter()
                    .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                {
                    self.spawn_tile(
                        parent_entity,
                        pos,
                        ObjectDefinition {
                            name: tile_name,
                            specifier: ObjectSpecifier::Furniture,
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
                        let Amount { obj: item, amount } = repetition.as_amount();
                        self.spawn_tile(
                            parent_entity,
                            pos,
                            ObjectDefinition {
                                name: tile_name,
                                specifier: ObjectSpecifier::Item(Item {
                                    amount: item.charges.unwrap_or(1) * amount,
                                }),
                            },
                        );
                    }
                }

                for spawn in submap
                    .spawns
                    .iter()
                    .filter(|spawn| spawn.x == x && spawn.z == z)
                {
                    self.spawn_tile(
                        parent_entity,
                        pos,
                        ObjectDefinition {
                            name: &spawn.spawn_type,
                            specifier: ObjectSpecifier::Character,
                        },
                    );
                }

                for fields in submap
                    .fields
                    .0
                    .iter()
                    .filter_map(|at| at.get(Pos::new(x, Level::ZERO, z)))
                {
                    for field in &fields.0 {
                        self.spawn_tile(
                            parent_entity,
                            pos,
                            ObjectDefinition {
                                name: &field.tile_name,
                                specifier: ObjectSpecifier::Terrain,
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
                    if let Some(name) = self.zone_level_names.get(zone_level) {
                        let definition = ObjectDefinition {
                            name: &name.clone(),
                            specifier: ObjectSpecifier::ZoneLevel,
                        };
                        self.spawn_collaped_zone_level(zone_level, &definition);
                    }
                }
            }
        }
    }

    fn spawn_collaped_zone_level(&mut self, zone_level: ZoneLevel, definition: &ObjectDefinition) {
        let pbr_bundles = match definition.name {
            name if name == &ObjectName::new("open_air")
                || name == &ObjectName::new("empty_rock") =>
            {
                Vec::new()
            }
            _ => {
                //println!("{zone_level:?} {:?}", &definition);
                self.loader
                    .get_models(definition)
                    .iter()
                    .map(|model| self.get_pbr_bundle(model))
                    .collect::<Vec<PbrBundle>>()
            }
        };

        let item_info = self.item_infos.get(definition.name);

        self.commands
            .spawn_bundle(SpatialBundle::default())
            .insert(
                item_info.map_or_else(|| definition.name.to_fallback_label(), |i| i.to_label(1)),
            )
            .insert(zone_level)
            .insert(Collapsed)
            .insert(Transform {
                translation: zone_level.base_pos().vec3() + Vec3::new(11.5, 0.0, 11.5),
                scale: Vec3::splat(24.0),
                ..Transform::default()
            })
            .insert(if self.explored.has_been_seen(zone_level) {
                LastSeen::Previously
            } else {
                LastSeen::Never
            })
            .with_children(|child_builder| {
                for pbr_bundle in pbr_bundles {
                    child_builder.spawn_bundle(pbr_bundle);
                }
            });
    }
}

pub(crate) struct CustomData {
    glass: Appearance,
    wood: Appearance,
    whitish: Appearance,
    wooden_wall: Appearance,
    yellow: Appearance,
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
            wooden_wall: Appearance::new(material_assets, asset_server.load("tiles/wall.png")),
            yellow: Appearance::new(material_assets, Color::rgb(0.8, 0.8, 0.4)),
            cube_mesh: mesh_assets.add(Mesh::from(shape::Cube { size: 1.0 })),
            wall_transform: Transform {
                translation: Vec3::new(0.0, 0.495 * VERTICAL.f32(), 0.0),
                scale: Vec3::new(ADJACENT.f32(), 0.99 * VERTICAL.f32(), ADJACENT.f32()),
                ..Transform::default()
            },
            window_pane_transform: Transform {
                translation: Vec3::new(0.0, 0.75, 0.0),
                rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                scale: Vec3::new(
                    0.99 * ADJACENT.f32(),
                    0.99 * VERTICAL.f32(),
                    0.99 * ADJACENT.f32(),
                ),
            },
            stair_transform: Transform {
                rotation: Quat::from_rotation_x(-0.12 * std::f32::consts::PI),
                scale: Vec3::new(0.8, 1.2 * VERTICAL.f32(), 0.2),
                ..Transform::default()
            },
            rack_transform: Transform {
                translation: Vec3::new(0.0, 0.45 * VERTICAL.f32(), 0.0),
                scale: Vec3::new(
                    0.90 * ADJACENT.f32(),
                    0.90 * VERTICAL.f32(),
                    0.90 * ADJACENT.f32(),
                ),
                ..default()
            },
            table_transform: Transform {
                translation: Vec3::new(0.0, 0.375 * ADJACENT.f32(), 0.0),
                scale: Vec3::new(ADJACENT.f32(), 0.75 * ADJACENT.f32(), ADJACENT.f32()),
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
            ObjectDefinition {
                name: &ObjectName::new("t_wood_stairs_down"),
                specifier: ObjectSpecifier::Terrain,
            },
        );
    }

    pub(crate) fn spawn_roofing(&mut self, parent: Entity, pos: Pos) {
        self.tile_spawner.spawn_tile(
            parent,
            pos,
            ObjectDefinition {
                name: &ObjectName::new("t_shingle_flat_roof"),
                specifier: ObjectSpecifier::Terrain,
            },
        );
    }

    pub(crate) fn spawn_wall(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Wall)
            .insert(pos)
            .insert(Integrity::new(1000))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(Label::new("wall"))
            .insert(LastSeen::Previously)
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
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
            .spawn_bundle(SpatialBundle::default())
            .insert(Window)
            .insert(pos)
            .insert(Integrity::new(1))
            .insert(Obstacle)
            .insert(Hurdle(2.5))
            .insert(Label::new("window"))
            .insert(LastSeen::Previously)
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wooden_wall.clone());

                parent
                    .spawn_bundle(PbrBundle {
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
            .spawn_bundle(SpatialBundle::default())
            .insert(StairsUp)
            .insert(pos)
            .insert(Integrity::new(100))
            .insert(Hurdle(1.5))
            .insert(Label::new("stairs"))
            .insert(LastSeen::Previously)
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(PbrBundle {
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
            .spawn_bundle(SpatialBundle::default())
            .insert(Rack)
            .insert(pos)
            .insert(Integrity::new(40))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(Label::new("rack"))
            .insert(LastSeen::Previously)
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
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
            .spawn_bundle(SpatialBundle::default())
            .insert(Table)
            .insert(pos)
            .insert(Integrity::new(30))
            .insert(Hurdle(2.0))
            .insert(Label::new("table"))
            .insert(LastSeen::Previously)
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: self.custom.table_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.wood.clone());
            });
    }

    pub(crate) fn spawn_chair(&mut self, pos: Pos) {
        let scale = 0.45 * ADJACENT.f32();

        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Chair)
            .insert(pos)
            .insert(Integrity::new(10))
            .insert(Hurdle(1.5))
            .insert(Label::new("chair"))
            .insert(LastSeen::Previously)
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(PbrBundle {
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
                    .spawn_bundle(PbrBundle {
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

    pub(crate) fn spawn_containable(&mut self, label: Label, pos: Pos, containable: Containable) {
        let size = f32::from(containable.0).min(64.0).powf(0.33) / 4.0;

        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(label)
            .insert(pos)
            .insert(containable)
            .insert(LastSeen::Previously)
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        // customizing the size using a transform allows better positioning
                        transform: Transform {
                            translation: Vec3::new(0.0, size / 2.0, 0.0),
                            scale: Vec3::splat(size),
                            ..Transform::default()
                        },
                        ..PbrBundle::default()
                    })
                    .insert(self.custom.yellow.clone());
            });
    }

    fn configure_player(&mut self, player_entity: Entity, player: Player) {
        let cursor_definition = ObjectDefinition {
            name: &ObjectName::new("cursor"),
            specifier: ObjectSpecifier::Meta,
        };
        let cursor_model = &mut self.tile_spawner.loader.get_models(&cursor_definition)[0];
        let mut cursor_bundle = self.tile_spawner.get_pbr_bundle(cursor_model);
        cursor_bundle.transform.translation.y = 0.1;
        cursor_bundle.transform.scale = Vec3::new(1.1, 1.0, 1.1);

        self.tile_spawner
            .commands
            .entity(player_entity)
            .insert(player)
            .insert(LevelChanged)
            .insert(ZoneChanged)
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(SpatialBundle::default())
                    .insert(CameraBase)
                    .with_children(|child_builder| {
                        child_builder
                            .spawn_bundle(cursor_bundle)
                            .insert(ExamineCursor);

                        let camera_direction = Transform::identity()
                            .looking_at(Vec3::new(1.0, 0.0, 0.1), Vec3::Y)
                            * Transform::from_translation(Vec3::new(0.0, 1.0, 0.0));
                        child_builder
                            .spawn_bundle(PbrBundle {
                                transform: camera_direction,
                                ..PbrBundle::default()
                            })
                            .with_children(|child_builder| {
                                child_builder.spawn_bundle(Camera3dBundle {
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
        speed: Speed,
        faction: Faction,
        player: Option<Player>,
    ) {
        let character_scale = 1.5;
        let character_definition = ObjectDefinition {
            name: &match faction {
                Faction::Human => ObjectName::new("overlay_male_mutation_SKIN_TAN"),
                _ => ObjectName::new("mon_zombie"),
            },
            specifier: ObjectSpecifier::Character,
        };
        let character_model = &mut self.tile_spawner.loader.get_models(&character_definition)[0];
        if let ModelShape::Plane {
            ref mut transform2d,
            ..
        } = character_model.shape
        {
            transform2d.scale *= character_scale;
        }
        let mut character_bundle = self.tile_spawner.get_pbr_bundle(character_model);
        character_bundle.transform.translation.y *= character_scale;
        println!("{:?}", character_bundle.transform);

        let character_appearance = self.tile_spawner.get_appearance(character_model);

        let entity = self
            .tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(label)
            .insert(pos)
            .insert(LastSeen::Previously)
            .insert(health)
            .insert(speed)
            .insert(faction)
            .insert(Obstacle)
            .insert(Container(4))
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(character_bundle)
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
        self.tile_spawner
            .commands
            .spawn_bundle(DirectionalLightBundle {
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

    pub(crate) fn spawn_floors(&mut self, offset: Pos) {
        let parent = self
            .tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
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

    pub(crate) fn spawn_house(&mut self, offset: Pos) {
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

    pub(crate) fn spawn_characters(&mut self, offset: Pos) {
        self.spawn_character(
            Label::new(self.tile_spawner.sav.player.name.clone()),
            Pos::new(45, Level::ZERO, 56).offset(offset).unwrap(),
            Health::new(10),
            Speed::from_h_kmph(6),
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
            Speed::from_h_kmph(7),
            Faction::Human,
            None,
        );
        self.spawn_character(
            Label::new("Zombie one"),
            Pos::new(12, Level::ZERO, 16).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(5),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie two"),
            Pos::new(40, Level::ZERO, 40).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(7),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie three"),
            Pos::new(38, Level::ZERO, 39).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(8),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie four"),
            Pos::new(37, Level::ZERO, 37).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(9),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie five"),
            Pos::new(34, Level::ZERO, 34).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(3),
            Faction::Zombie,
            None,
        );
    }

    pub(crate) fn spawn_containables(&mut self, offset: Pos) {
        self.spawn_containable(
            Label::new("large"),
            Pos::new(13, Level::ZERO, 13).offset(offset).unwrap(),
            Containable(4),
        );
        self.spawn_containable(
            Label::new("small"),
            Pos::new(13, Level::ZERO, 12).offset(offset).unwrap(),
            Containable(1),
        );
        self.spawn_containable(
            Label::new("medium"),
            Pos::new(13, Level::ZERO, 10).offset(offset).unwrap(),
            Containable(2),
        );
    }

    pub(crate) fn spawn_window_wall(&mut self, offset: Pos) {
        for i in 0..48 {
            self.spawn_window(Pos::new(i, Level::ZERO, 15).offset(offset).unwrap());
        }
    }
}
