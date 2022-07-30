use crate::prelude::*;
use bevy::ecs::system::{Insert, Remove, SystemParam};
use bevy::math::Quat;
use bevy::prelude::*;
use bevy::render::camera::{PerspectiveProjection, Projection::Perspective};
use bevy::utils::HashMap;

#[derive(PartialEq)]
pub enum TileType {
    Terrain,
    Furniture,
    Item(Item),
    Character,
    ZoneLayer,
    Meta,
}

pub struct TileCaches {
    material_cache: HashMap<String, Handle<StandardMaterial>>,
    plane_mesh_cache: HashMap<SpriteNumber, Handle<Mesh>>,
    cuboid_mesh_cache: HashMap<SpriteNumber, Handle<Mesh>>,
}

impl TileCaches {
    pub fn new() -> Self {
        Self {
            material_cache: HashMap::new(),
            plane_mesh_cache: HashMap::new(),
            cuboid_mesh_cache: HashMap::new(),
        }
    }
}

#[derive(SystemParam)]
pub struct TileSpawner<'w, 's> {
    commands: Commands<'w, 's>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    asset_server: Res<'w, AssetServer>,
    loader: Res<'w, TileLoader>,
    caches: ResMut<'w, TileCaches>,
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

    fn get_tile_material(
        &mut self,
        imagepath: &str,
        alpha_mode: AlphaMode,
    ) -> Handle<StandardMaterial> {
        self.caches
            .material_cache
            .entry(imagepath.to_string())
            .or_insert_with(|| {
                self.material_assets.add(StandardMaterial {
                    base_color_texture: Some(self.asset_server.load(imagepath)),
                    alpha_mode,
                    ..StandardMaterial::default()
                })
            })
            .clone()
    }

    fn get_pbr_bundle(
        &mut self,
        tile_name: &TileName,
        model: &Model,
        tile_type: &TileType,
    ) -> PbrBundle {
        let alpha_mode = if tile_type == &TileType::Terrain && tile_name.is_ground() {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        };
        PbrBundle {
            mesh: self.get_tile_mesh(model),
            material: self.get_tile_material(&model.texture_path, alpha_mode),
            transform: model.to_transform(),
            ..PbrBundle::default()
        }
    }

    fn spawn_tile(&mut self, parent: Entity, pos: Pos, tile_name: &TileName, tile_type: TileType) {
        let models = self.loader.get_models(tile_name, &tile_type);
        let pbr_bundles = models
            .iter()
            .map(|model| self.get_pbr_bundle(tile_name, model, &tile_type))
            .collect::<Vec<PbrBundle>>();

        self.commands.entity(parent).with_children(|child_builder| {
            let tile = child_builder
                .spawn_bundle(SpatialBundle::default())
                .insert(tile_name.to_label())
                .insert(pos)
                .insert(Transform::from_translation(pos.vec3()))
                .with_children(|child_builder| {
                    for pbr_bundle in pbr_bundles {
                        child_builder.spawn_bundle(pbr_bundle.clone());
                    }
                })
                .id();

            match tile_type {
                TileType::Character => {
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
                TileType::Item(item) => {
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: item,
                    });
                }
                _ => {
                    if tile_type == TileType::Terrain {
                        child_builder.add_command(Insert {
                            entity: tile,
                            component: Floor,
                        });
                    }

                    match models
                        .iter()
                        .find(|m| m.layer == SpriteLayer::Front)
                        .unwrap()
                        .shape
                    {
                        ModelShape::Plane {
                            orientation,
                            transform2d,
                        } => {
                            if tile_name.is_stairs_up() {
                                child_builder.add_command(Insert {
                                    entity: tile,
                                    component: Stairs,
                                });
                            } else if orientation == SpriteOrientation::Vertical {
                                match transform2d.scale.y {
                                    x if 2.5 < x => {
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
                                    x if 1.5 < x => {
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
            }
        });
    }

    pub fn spawn_expanded_zone_level(&mut self, zone_level: ZoneLevel) -> Result<(), ()> {
        if let Ok(map) = Map::try_from(zone_level) {
            let zone_level_entity = self
                .commands
                .spawn_bundle(SpatialBundle::default())
                .insert(zone_level)
                .id();
            let base = map.submaps[0].coordinates;
            for submap in &map.submaps {
                assert!(base.2 == submap.coordinates.2);
                self.spawn_subzone(
                    submap,
                    zone_level_entity,
                    zone_level,
                    Pos(
                        12 * (submap.coordinates.0 - base.0),
                        0,
                        12 * (submap.coordinates.1 - base.1),
                    ),
                );
            }
            Ok(())
        } else {
            Err(())
        }
    }

    fn spawn_subzone(
        &mut self,
        submap: &Submap,
        parent_entity: Entity,
        zone_level: ZoneLevel,
        subzone_offset: Pos,
    ) {
        for x in 0..12 {
            for z in 0..12 {
                let pos = zone_level
                    .base_pos()
                    .offset(subzone_offset)
                    .unwrap()
                    .offset(Pos(x, 0, z))
                    .unwrap();
                let tile_name = &submap.terrain[(x + 12 * z) as usize];
                if tile_name != &TileName::new("t_open_air")
                    && tile_name != &TileName::new("t_open_air_rooved")
                {
                    self.spawn_tile(parent_entity, pos, tile_name, TileType::Terrain);
                }

                for tile_name in submap
                    .furniture
                    .iter()
                    .filter_map(|at| at.get(Pos(x, 0, z)))
                {
                    self.spawn_tile(parent_entity, pos, tile_name, TileType::Furniture);
                }

                for repetitions in submap.items.iter().filter_map(|at| at.get(Pos(x, 0, z))) {
                    for repetition in repetitions {
                        let Repetition { obj: item, amount } = repetition;
                        self.spawn_tile(
                            parent_entity,
                            pos,
                            &item.typeid,
                            TileType::Item(Item {
                                amount: item.charges.unwrap_or(1) * amount,
                            }),
                        );
                    }
                }

                for spawn in submap
                    .spawns
                    .iter()
                    .filter(|spawn| spawn.x == x && spawn.z == z)
                {
                    self.spawn_tile(parent_entity, pos, &spawn.spawn_type, TileType::Character);
                }

                for field in submap.fields.iter().filter_map(|at| at.get(Pos(x, 0, z))) {
                    self.spawn_tile(parent_entity, pos, &field.tile_name, TileType::Terrain);
                }
            }
        }
    }

    pub fn spawn_overmaps(&mut self) {
        // TODO Which hierarchy?
        // - Overmap - Zone - ZoneLevel - Pos
        // - Overmap - OvermapLevel - ZoneLevel - Pos

        /*
        let zone_level_entity = self
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(zone_level)
            .id();
        self.commands
            .entity(zone_entity)
            .add_child(zone_level_entity);
        */

        for x in 0..=0 {
            for z in 1..=1 {
                let overzone = Overzone { x, z };
                if let Ok(overmap) = Overmap::try_from(overzone) {
                    let overzone_level = overzone.overzone_level(0);
                    self.spawn_overmap_levels(
                        &overmap.layers[overzone_level.level_index()],
                        overzone_level,
                    );
                }
            }
        }
    }

    fn spawn_overmap_levels(
        &mut self,
        overmap_layer: &OvermapLevel,
        overzone_level: OverzoneLevel,
    ) {
        let mut i: i32 = 0;
        for (tile_name, amount) in &overmap_layer.0 {
            let amount = i32::from(*amount);
            for j in i..i + amount {
                let x = j.rem_euclid(180);
                let z = j.div_euclid(180);
                let zone_level = overzone_level.base_zone_level().offset(x, z);
                println!("{zone_level:?}");
                self.spawn_collaped_zone_level(zone_level, tile_name);
            }
            i += amount;
        }
        assert!(i == 32400, "{i}");
    }

    fn spawn_collaped_zone_level(&mut self, zone_level: ZoneLevel, tile_name: &TileName) {
        let tile_type = &TileType::ZoneLayer;
        let pbr_bundles = self
            .loader
            .get_models(tile_name, tile_type)
            .iter()
            .map(|model| self.get_pbr_bundle(tile_name, model, tile_type))
            .collect::<Vec<PbrBundle>>();

        self.commands
            .spawn_bundle(SpatialBundle::default())
            .insert(tile_name.to_label())
            .insert(zone_level)
            .insert(Collapsed)
            .insert(Transform {
                translation: zone_level.base_pos().vec3() + Vec3::new(11.5, 0.0, 11.5),
                scale: Vec3::splat(24.0),
                ..Transform::default()
            })
            .with_children(|child_builder| {
                for pbr_bundle in pbr_bundles {
                    child_builder.spawn_bundle(pbr_bundle);
                }
            });
    }
}

pub struct CustomData {
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
    pub fn new(
        material_assets: &mut Assets<StandardMaterial>,
        mesh_assets: &mut Assets<Mesh>,
        asset_server: &Res<AssetServer>,
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
                rotation: Quat::from_rotation_y(-0.5 * std::f32::consts::PI),
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
pub struct Spawner<'w, 's> {
    tile_spawner: TileSpawner<'w, 's>,
    custom: ResMut<'w, CustomData>,
}

impl<'w, 's> Spawner<'w, 's> {
    pub fn spawn_stairs_down(&mut self, parent: Entity, pos: Pos) {
        self.tile_spawner.spawn_tile(
            parent,
            pos,
            &TileName::new("t_wood_stairs_down"),
            TileType::Terrain,
        );
    }

    pub fn spawn_roofing(&mut self, parent: Entity, pos: Pos) {
        self.tile_spawner.spawn_tile(
            parent,
            pos,
            &TileName::new("t_shingle_flat_roof"),
            TileType::Terrain,
        );
    }

    pub fn spawn_wall(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Wall)
            .insert(pos)
            .insert(Integrity::new(1000))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(Label::new("wall"))
            .insert(PlayerVisible::Reevaluate)
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

    pub fn spawn_window(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Window)
            .insert(pos)
            .insert(Integrity::new(1))
            .insert(Obstacle)
            .insert(Hurdle(2.5))
            .insert(Label::new("window"))
            .insert(PlayerVisible::Reevaluate)
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

    pub fn spawn_stairs(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Stairs)
            .insert(pos)
            .insert(Integrity::new(100))
            .insert(Hurdle(1.5))
            .insert(Label::new("stairs"))
            .insert(PlayerVisible::Reevaluate)
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

    pub fn spawn_rack(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Rack)
            .insert(pos)
            .insert(Integrity::new(40))
            .insert(Obstacle)
            .insert(Opaque)
            .insert(Label::new("rack"))
            .insert(PlayerVisible::Reevaluate)
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

    pub fn spawn_table(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Table)
            .insert(pos)
            .insert(Integrity::new(30))
            .insert(Hurdle(2.0))
            .insert(Label::new("table"))
            .insert(PlayerVisible::Reevaluate)
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

    pub fn spawn_chair(&mut self, pos: Pos) {
        let scale = 0.45 * ADJACENT.f32();

        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(Chair)
            .insert(pos)
            .insert(Integrity::new(10))
            .insert(Hurdle(1.5))
            .insert(Label::new("chair"))
            .insert(PlayerVisible::Reevaluate)
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

    pub fn spawn_containable(&mut self, label: Label, pos: Pos, containable: Containable) {
        let size = f32::from(containable.0).min(64.0).powf(0.33) / 4.0;

        self.tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(label)
            .insert(pos)
            .insert(containable)
            .insert(PlayerVisible::Reevaluate)
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
        let cursor_tile_type = &TileType::Meta;
        let cursor_tile_name = TileName::new("cursor");
        let cursor_model = &mut self
            .tile_spawner
            .loader
            .get_models(&cursor_tile_name, cursor_tile_type)[0];
        let mut cursor_bundle =
            self.tile_spawner
                .get_pbr_bundle(&cursor_tile_name, cursor_model, cursor_tile_type);
        cursor_bundle.transform.translation.y = 0.15;
        cursor_bundle.transform.scale = Vec3::new(1.15, 1.0, 1.15);

        self.tile_spawner
            .commands
            .entity(player_entity)
            .insert(player)
            .insert(PosYChanged)
            .insert(ZoneChanged)
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(SpatialBundle::default())
                    .insert(CameraBase)
                    .with_children(|child_builder| {
                        child_builder
                            .spawn_bundle(cursor_bundle)
                            .insert(ExamineCursor);

                        let camera_direction =
                            Transform::identity().looking_at(Vec3::new(1.0, 0.0, 0.1), Vec3::Y);
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

    pub fn spawn_character(
        &mut self,
        label: Label,
        pos: Pos,
        health: Health,
        speed: Speed,
        faction: Faction,
        player: Option<Player>,
    ) {
        let character_scale = 1.5;
        let character_tile_type = &TileType::Character;
        let character_tile_name = match faction {
            Faction::Human => TileName::new("overlay_male_mutation_SKIN_TAN"),
            _ => TileName::new("mon_zombie"),
        };
        let character_sprite_info = &mut self
            .tile_spawner
            .loader
            .get_models(&character_tile_name, character_tile_type)[0];
        if let ModelShape::Plane {
            ref mut transform2d,
            ..
        } = character_sprite_info.shape
        {
            transform2d.scale *= character_scale;
        }
        let mut character_bundle = self.tile_spawner.get_pbr_bundle(
            &character_tile_name,
            character_sprite_info,
            character_tile_type,
        );
        character_bundle.transform.translation.y *= character_scale;
        println!("{:?}", character_bundle.transform);

        let entity = self
            .tile_spawner
            .commands
            .spawn_bundle(SpatialBundle::default())
            .insert(label)
            .insert(pos)
            .insert(health)
            .insert(speed)
            .insert(faction)
            .insert(Obstacle)
            .insert(Container(4))
            .with_children(|child_builder| {
                child_builder.spawn_bundle(character_bundle);
            })
            .id();

        if let Some(player) = player {
            self.configure_player(entity, player);
        }
    }

    pub fn spawn_light(&mut self) {
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

    pub fn spawn_floors(&mut self, offset: Pos) {
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
                    let pos = Pos(x, y, z);
                    match pos {
                        Pos(18, 1, 24) | Pos(18, 2, 22) => {
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

    pub fn spawn_house(&mut self, offset: Pos) {
        self.spawn_wall(Pos(17, 0, 17).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 0, 18).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 0, 19).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 0, 20).offset(offset).unwrap());
        // z 21: door
        self.spawn_wall(Pos(17, 0, 22).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 0, 23).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 0, 24).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 0, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(18, 0, 25).offset(offset).unwrap());
        self.spawn_window(Pos(19, 0, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(20, 0, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(21, 0, 25).offset(offset).unwrap());
        // x 22: door
        self.spawn_wall(Pos(23, 0, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 0, 24).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 0, 23).offset(offset).unwrap());
        self.spawn_window(Pos(23, 0, 22).offset(offset).unwrap());
        self.spawn_window(Pos(23, 0, 21).offset(offset).unwrap());
        self.spawn_window(Pos(23, 0, 20).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 0, 19).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 0, 18).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 0, 17).offset(offset).unwrap());
        self.spawn_wall(Pos(22, 0, 17).offset(offset).unwrap());
        self.spawn_wall(Pos(21, 0, 17).offset(offset).unwrap());
        self.spawn_window(Pos(20, 0, 17).offset(offset).unwrap());
        self.spawn_wall(Pos(19, 0, 17).offset(offset).unwrap());
        self.spawn_wall(Pos(18, 0, 17).offset(offset).unwrap());

        self.spawn_stairs(Pos(18, 0, 24).offset(offset).unwrap());
        self.spawn_rack(Pos(18, 0, 18).offset(offset).unwrap());
        self.spawn_rack(Pos(20, 0, 24).offset(offset).unwrap());
        self.spawn_chair(Pos(19, 0, 20).offset(offset).unwrap());
        self.spawn_table(Pos(19, 0, 21).offset(offset).unwrap());
        self.spawn_chair(Pos(19, 0, 22).offset(offset).unwrap());
        self.spawn_chair(Pos(20, 0, 20).offset(offset).unwrap());
        self.spawn_table(Pos(20, 0, 21).offset(offset).unwrap());
        self.spawn_chair(Pos(20, 0, 22).offset(offset).unwrap());
        self.spawn_table(Pos(22, 0, 22).offset(offset).unwrap());
        self.spawn_table(Pos(22, 0, 21).offset(offset).unwrap());
        self.spawn_table(Pos(22, 0, 20).offset(offset).unwrap());
        self.spawn_table(Pos(22, 0, 19).offset(offset).unwrap());
        self.spawn_table(Pos(22, 0, 18).offset(offset).unwrap());
        self.spawn_table(Pos(21, 0, 18).offset(offset).unwrap());
        self.spawn_table(Pos(20, 0, 18).offset(offset).unwrap());

        self.spawn_wall(Pos(17, 1, 21).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 1, 22).offset(offset).unwrap());
        self.spawn_window(Pos(17, 1, 23).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 1, 24).offset(offset).unwrap());
        self.spawn_wall(Pos(17, 1, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(18, 1, 25).offset(offset).unwrap());
        self.spawn_window(Pos(19, 1, 25).offset(offset).unwrap());
        self.spawn_window(Pos(20, 1, 25).offset(offset).unwrap());
        self.spawn_window(Pos(21, 1, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(22, 1, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 1, 25).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 1, 24).offset(offset).unwrap());
        self.spawn_window(Pos(23, 1, 23).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 1, 22).offset(offset).unwrap());
        self.spawn_wall(Pos(23, 1, 21).offset(offset).unwrap());
        self.spawn_wall(Pos(22, 1, 21).offset(offset).unwrap());
        self.spawn_wall(Pos(21, 1, 21).offset(offset).unwrap());
        // x 20: door
        self.spawn_wall(Pos(19, 1, 21).offset(offset).unwrap());
        self.spawn_wall(Pos(18, 1, 21).offset(offset).unwrap());

        self.spawn_stairs(Pos(18, 1, 22).offset(offset).unwrap());
        self.spawn_table(Pos(21, 1, 24).offset(offset).unwrap());
        self.spawn_table(Pos(22, 1, 24).offset(offset).unwrap());
        self.spawn_table(Pos(22, 1, 23).offset(offset).unwrap());
        self.spawn_table(Pos(22, 1, 22).offset(offset).unwrap());
        self.spawn_chair(Pos(21, 1, 23).offset(offset).unwrap());
    }

    pub fn spawn_characters(&mut self, offset: Pos) {
        self.spawn_character(
            Label::new("T"),
            Pos(45, 0, 56).offset(offset).unwrap(),
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
            Pos(10, 0, 10).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(7),
            Faction::Human,
            None,
        );
        self.spawn_character(
            Label::new("Zombie one"),
            Pos(12, 0, 16).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(5),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie two"),
            Pos(40, 0, 40).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(7),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie three"),
            Pos(38, 0, 39).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(8),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie four"),
            Pos(37, 0, 37).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(9),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie five"),
            Pos(34, 0, 34).offset(offset).unwrap(),
            Health::new(3),
            Speed::from_h_kmph(3),
            Faction::Zombie,
            None,
        );
    }

    pub fn spawn_containables(&mut self, offset: Pos) {
        self.spawn_containable(
            Label::new("large"),
            Pos(13, 0, 13).offset(offset).unwrap(),
            Containable(4),
        );
        self.spawn_containable(
            Label::new("small"),
            Pos(13, 0, 12).offset(offset).unwrap(),
            Containable(1),
        );
        self.spawn_containable(
            Label::new("medium"),
            Pos(13, 0, 10).offset(offset).unwrap(),
            Containable(2),
        );
    }

    pub fn spawn_window_wall(&mut self, offset: Pos) {
        for i in 0..48 {
            self.spawn_window(Pos(i, 0, 15).offset(offset).unwrap());
        }
    }
}
