use bevy::ecs::system::{Insert, Remove, SystemParam};
use bevy::math::Quat;
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy::utils::HashMap;

use super::tile_loader::{SpriteLayer, SpriteNumber, TileLoader, TileName};
use super::zone_loader::{zone_layout, SubzoneLayout, ZoneLayout};
use crate::components::Window;
use crate::components::{
    Appearance, CameraBase, Chair, Containable, Container, ExamineCursor, Faction, Floor, Health,
    Hurdle, Integrity, Label, Obstacle, Opaque, Player, PlayerActionState, PlayerVisible, Pos,
    PosYChanged, Rack, Stairs, Table, Wall, WindowPane, Zone, ZoneChanged, ZoneLevel,
};
use crate::model::{Model, ModelShape, SpriteOrientation};
use crate::unit::{Speed, ADJACENT, VERTICAL};

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Terrain,
    Furniture,
    Item,
    Character,
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
        tile_type: TileType,
    ) -> PbrBundle {
        let alpha_mode = if tile_type == TileType::Terrain && tile_name.is_ground() {
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
        let models = self.loader.get_models(tile_name);
        let pbr_bundles = models
            .iter()
            .map(|model| self.get_pbr_bundle(tile_name, model, tile_type))
            .collect::<Vec<PbrBundle>>();

        self.commands.entity(parent).with_children(|child_builder| {
            let tile = child_builder
                .spawn_bundle((
                    pos,
                    tile_name.to_label(),
                    Transform::from_translation(pos.vec3()),
                    GlobalTransform::default(),
                    Visibility::default(),
                ))
                .with_children(|child_builder| {
                    for pbr_bundle in pbr_bundles {
                        child_builder.spawn_bundle(pbr_bundle);
                    }
                })
                .id();

            if tile_type == TileType::Character {
                child_builder.add_command(Insert {
                    entity: tile,
                    component: Obstacle,
                });
                child_builder.add_command(Insert {
                    entity: tile,
                    component: Health::new(5),
                });
            } else {
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
                        if tile_type == TileType::Terrain {
                            child_builder.add_command(Insert {
                                entity: tile,
                                component: Floor,
                            });
                        }

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
        });
    }

    pub fn load_cdda_region(&mut self, base_zone: Zone, size: i32) {
        for x in 0..size {
            for z in 0..size {
                let zone = base_zone.offset(x, z);
                let zone_entity = self
                    .commands
                    .spawn_bundle(PbrBundle::default())
                    .insert(zone)
                    .id();
                for y in Pos::vertical_range() {
                    let zone_level = zone.zone_level(y);
                    /*
                    TODO wait for https://github.com/bevyengine/bevy/pull/2087
                    let zone_level_entity = self
                        .commands
                        .spawn_bundle(PbrBundle::default())
                        .insert(zone_level)
                        .id();
                    self.commands
                        .entity(zone_entity)
                        .add_child(zone_level_entity);
                    */
                    if let Some(zone_layout) = zone_layout(zone_level.offset(100, 212)) {
                        self.spawn_zone(&zone_layout, zone_entity, zone_level);
                    }
                }
            }
        }
    }

    fn spawn_zone(
        &mut self,
        zone_layout: &ZoneLayout,
        parent_entity: Entity,
        zone_level: ZoneLevel,
    ) {
        let base = zone_layout.subzone_layouts[0].coordinates;
        for subzone_layout in &zone_layout.subzone_layouts {
            assert!(base.2 == subzone_layout.coordinates.2);
            self.spawn_subzone(
                subzone_layout,
                parent_entity,
                zone_level,
                Pos(
                    12 * (subzone_layout.coordinates.0 - base.0),
                    0,
                    12 * (subzone_layout.coordinates.1 - base.1),
                ),
            );
        }
    }

    fn spawn_subzone(
        &mut self,
        subzone_layout: &SubzoneLayout,
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
                let tile_name = &subzone_layout.terrain[(x + 12 * z) as usize];
                if tile_name != &TileName::new("t_open_air")
                    && tile_name != &TileName::new("t_open_air_rooved")
                {
                    self.spawn_tile(parent_entity, pos, tile_name, TileType::Terrain);
                }

                for tile_name in subzone_layout
                    .furniture
                    .iter()
                    .filter_map(|at| at.get(Pos(x, 0, z)))
                {
                    self.spawn_tile(parent_entity, pos, tile_name, TileType::Furniture);
                }

                for item in subzone_layout
                    .items
                    .iter()
                    .filter_map(|at| at.get(Pos(x, 0, z)))
                {
                    self.spawn_tile(parent_entity, pos, &item.typeid, TileType::Item);
                }

                for spawn in subzone_layout
                    .spawns
                    .iter()
                    .filter(|spawn| spawn.x == x && spawn.z == z)
                {
                    self.spawn_tile(parent_entity, pos, &spawn.spawn_type, TileType::Character);
                }
            }
        }
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
            wall_transform: Transform::from_scale(Vec3::new(
                ADJACENT.f32(),
                0.99 * VERTICAL.f32(),
                ADJACENT.f32(),
            )),
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
            rack_transform: Transform::from_scale(Vec3::new(
                0.90 * ADJACENT.f32(),
                0.90 * VERTICAL.f32(),
                0.90 * ADJACENT.f32(),
            )),
            table_transform: Transform::from_scale(Vec3::new(
                ADJACENT.f32(),
                0.75 * ADJACENT.f32(),
                ADJACENT.f32(),
            )),
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
            .spawn_bundle((
                Wall,
                pos,
                Integrity::new(1000),
                Obstacle,
                Opaque,
                self.custom.wooden_wall.clone(),
                Label::new("wall"),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.custom.cube_mesh.clone(),
                transform: self.custom.wall_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_window(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle((
                Window,
                pos,
                Integrity::new(1),
                Obstacle,
                Hurdle(2.5),
                Label::new("window"),
                self.custom.wooden_wall.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.custom.cube_mesh.clone(),
                ..PbrBundle::default()
            })
            .with_children(|child_builder| {
                child_builder
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
            .spawn_bundle((
                Stairs,
                pos,
                Integrity::new(100),
                Hurdle(1.5),
                Label::new("stairs"),
                self.custom.wood.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.custom.cube_mesh.clone(),
                transform: self.custom.stair_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_rack(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle((
                Rack,
                pos,
                Integrity::new(40),
                Obstacle,
                Opaque,
                Label::new("rack"),
                self.custom.wood.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.custom.cube_mesh.clone(),
                transform: self.custom.rack_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_table(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle((
                Table,
                pos,
                Integrity::new(30),
                Hurdle(2.0),
                Label::new("table"),
                self.custom.wood.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.custom.cube_mesh.clone(),
                transform: self.custom.table_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_chair(&mut self, pos: Pos) {
        self.tile_spawner
            .commands
            .spawn_bundle((
                Chair,
                pos,
                Integrity::new(10),
                Hurdle(1.5),
                Label::new("chair"),
                self.custom.whitish.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.custom.cube_mesh.clone(),
                transform: Transform::from_scale(Vec3::new(
                    0.45 * ADJACENT.f32(),
                    0.45 * ADJACENT.f32(),
                    0.45 * ADJACENT.f32(),
                )),
                ..PbrBundle::default()
            })
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(PbrBundle {
                        mesh: self.custom.cube_mesh.clone(),
                        transform: Transform {
                            translation: Vec3::new(0.0, 0.2 / 0.45, -0.23 / 0.45),
                            rotation: Quat::from_rotation_y(-0.5 * std::f32::consts::PI),
                            scale: Vec3::new(0.05 / 0.45, 0.85 / 0.45, 1.0),
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
            .spawn_bundle((
                label,
                pos,
                containable,
                self.custom.yellow.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle::default())
            .with_children(|child_builder| {
                child_builder.spawn_bundle(PbrBundle {
                    mesh: self.custom.cube_mesh.clone(),
                    // customizing the size using a transform allows better positioning
                    transform: Transform {
                        translation: Vec3::new(0.0, size, 0.0),
                        scale: Vec3::new(size, size, size),
                        ..Transform::default()
                    },
                    ..PbrBundle::default()
                });
            });
    }

    fn configure_player(&mut self, player_entity: Entity, player: Player) {
        let cursor_tile_name = TileName::new("cursor");
        let cursor_model = &mut self.tile_spawner.loader.get_models(&cursor_tile_name)[0];
        let mut cursor_bundle =
            self.tile_spawner
                .get_pbr_bundle(&cursor_tile_name, cursor_model, TileType::Meta);
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
                    .spawn_bundle(PbrBundle::default())
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
                                child_builder.spawn_bundle(PerspectiveCameraBundle {
                                    perspective_projection: PerspectiveProjection {
                                        // more overview, less personal than the default
                                        fov: 0.3,
                                        ..PerspectiveProjection::default()
                                    },
                                    ..PerspectiveCameraBundle::default()
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

        let character_tile_name = match faction {
            Faction::Human => TileName::new("overlay_male_mutation_SKIN_TAN"),
            Faction::Zombie => TileName::new("mon_zombie"),
        };
        let character_sprite_info =
            &mut self.tile_spawner.loader.get_models(&character_tile_name)[0];
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
            TileType::Character,
        );
        character_bundle.transform.translation.y *= character_scale;
        println!("{:?}", character_bundle.transform);

        let entity = self
            .tile_spawner
            .commands
            .spawn_bundle((label, pos, health, speed, faction, Obstacle, Container(4)))
            .insert_bundle(PbrBundle::default())
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

    pub fn spawn_floors(&mut self) {
        let parent = self
            .tile_spawner
            .commands
            .spawn_bundle(PbrBundle::default())
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
                            self.spawn_stairs_down(parent, pos);
                        }
                        _ => {
                            self.spawn_roofing(parent, pos);
                        }
                    }
                }
            }
        }
    }

    pub fn spawn_house(&mut self) {
        self.spawn_wall(Pos(17, 0, 17));
        self.spawn_wall(Pos(17, 0, 18));
        self.spawn_wall(Pos(17, 0, 19));
        self.spawn_wall(Pos(17, 0, 20));
        // z 21: door
        self.spawn_wall(Pos(17, 0, 22));
        self.spawn_wall(Pos(17, 0, 23));
        self.spawn_wall(Pos(17, 0, 24));
        self.spawn_wall(Pos(17, 0, 25));
        self.spawn_wall(Pos(18, 0, 25));
        self.spawn_window(Pos(19, 0, 25));
        self.spawn_wall(Pos(20, 0, 25));
        self.spawn_wall(Pos(21, 0, 25));
        // x 22: door
        self.spawn_wall(Pos(23, 0, 25));
        self.spawn_wall(Pos(23, 0, 24));
        self.spawn_wall(Pos(23, 0, 23));
        self.spawn_window(Pos(23, 0, 22));
        self.spawn_window(Pos(23, 0, 21));
        self.spawn_window(Pos(23, 0, 20));
        self.spawn_wall(Pos(23, 0, 19));
        self.spawn_wall(Pos(23, 0, 18));
        self.spawn_wall(Pos(23, 0, 17));
        self.spawn_wall(Pos(22, 0, 17));
        self.spawn_wall(Pos(21, 0, 17));
        self.spawn_window(Pos(20, 0, 17));
        self.spawn_wall(Pos(19, 0, 17));
        self.spawn_wall(Pos(18, 0, 17));

        self.spawn_stairs(Pos(18, 0, 24));
        self.spawn_rack(Pos(18, 0, 18));
        self.spawn_rack(Pos(20, 0, 24));
        self.spawn_chair(Pos(19, 0, 20));
        self.spawn_table(Pos(19, 0, 21));
        self.spawn_chair(Pos(19, 0, 22));
        self.spawn_chair(Pos(20, 0, 20));
        self.spawn_table(Pos(20, 0, 21));
        self.spawn_chair(Pos(20, 0, 22));
        self.spawn_table(Pos(22, 0, 22));
        self.spawn_table(Pos(22, 0, 21));
        self.spawn_table(Pos(22, 0, 20));
        self.spawn_table(Pos(22, 0, 19));
        self.spawn_table(Pos(22, 0, 18));
        self.spawn_table(Pos(21, 0, 18));
        self.spawn_table(Pos(20, 0, 18));

        self.spawn_wall(Pos(17, 1, 21));
        self.spawn_wall(Pos(17, 1, 22));
        self.spawn_window(Pos(17, 1, 23));
        self.spawn_wall(Pos(17, 1, 24));
        self.spawn_wall(Pos(17, 1, 25));
        self.spawn_wall(Pos(18, 1, 25));
        self.spawn_window(Pos(19, 1, 25));
        self.spawn_window(Pos(20, 1, 25));
        self.spawn_window(Pos(21, 1, 25));
        self.spawn_wall(Pos(22, 1, 25));
        self.spawn_wall(Pos(23, 1, 25));
        self.spawn_wall(Pos(23, 1, 24));
        self.spawn_window(Pos(23, 1, 23));
        self.spawn_wall(Pos(23, 1, 22));
        self.spawn_wall(Pos(23, 1, 21));
        self.spawn_wall(Pos(22, 1, 21));
        self.spawn_wall(Pos(21, 1, 21));
        // x 20: door
        self.spawn_wall(Pos(19, 1, 21));
        self.spawn_wall(Pos(18, 1, 21));

        self.spawn_stairs(Pos(18, 1, 22));
        self.spawn_table(Pos(21, 1, 24));
        self.spawn_table(Pos(22, 1, 24));
        self.spawn_table(Pos(22, 1, 23));
        self.spawn_table(Pos(22, 1, 22));
        self.spawn_chair(Pos(21, 1, 23));
    }

    pub fn spawn_characters(&mut self) {
        self.spawn_character(
            Label::new("T"),
            Pos(1, 0, 1), // TODO Pos(45, 0, 45),
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
            Pos(10, 0, 10),
            Health::new(3),
            Speed::from_h_kmph(7),
            Faction::Human,
            None,
        );
        self.spawn_character(
            Label::new("Zombie one"),
            Pos(12, 0, 16),
            Health::new(3),
            Speed::from_h_kmph(5),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie two"),
            Pos(40, 0, 40),
            Health::new(3),
            Speed::from_h_kmph(7),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie three"),
            Pos(38, 0, 39),
            Health::new(3),
            Speed::from_h_kmph(8),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie four"),
            Pos(37, 0, 37),
            Health::new(3),
            Speed::from_h_kmph(9),
            Faction::Zombie,
            None,
        );
        self.spawn_character(
            Label::new("Zombie five"),
            Pos(34, 0, 34),
            Health::new(3),
            Speed::from_h_kmph(3),
            Faction::Zombie,
            None,
        );
    }

    pub fn spawn_containables(&mut self) {
        self.spawn_containable(Label::new("large"), Pos(13, 0, 13), Containable(4));
        self.spawn_containable(Label::new("small"), Pos(13, 0, 12), Containable(1));
        self.spawn_containable(Label::new("medium"), Pos(13, 0, 10), Containable(2));
    }

    pub fn spawn_window_wall(&mut self) {
        for i in 0..48 {
            self.spawn_window(Pos(i, 0, 15));
        }
    }
}
