use bevy::ecs::system::{Insert, Remove, SystemParam};
use bevy::math::Quat;
use bevy::prelude::*;
use bevy::render::{
    camera::PerspectiveProjection,
    mesh::{Indices, PrimitiveTopology},
};
use bevy::utils::HashMap;

use super::super::components::Window;
use super::super::components::{
    Appearance, CameraBase, CameraCursor, Chair, Containable, Container, Faction, Floor, Health,
    Hurdle, Integrity, Label, LogDisplay, ManualDisplay, Obstacle, Opaque, Player,
    PlayerActionState, PlayerVisible, Pos, PosYChanged, Rack, Stairs, StatusDisplay, Table, Wall,
    WindowPane, Zone, ZoneChanged, ZoneLevel,
};
use super::super::units::{Speed, ADJACENT, VERTICAL};
use super::tile_loader::{
    MeshInfo, SpriteInfo, SpriteLayer, SpriteOrientation, TileLoader, TileName,
};
use super::zone_loader::{zone_layout, SubzoneLayout, ZoneLayout};

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Terrain,
    Furniture,
    Item,
    Character,
}

pub struct SpawnerData {
    character: Appearance,
    glass: Appearance,
    wood: Appearance,
    whitish: Appearance,
    wooden_wall: Appearance,
    yellow: Appearance,
    tile_materials: HashMap<String, Handle<StandardMaterial>>,
    character_mesh: Handle<Mesh>,
    cube_mesh: Handle<Mesh>,
    tile_meshes: HashMap<MeshInfo, Handle<Mesh>>,
    wall_transform: Transform,
    window_pane_transform: Transform,
    stair_transform: Transform,
    rack_transform: Transform,
    table_transform: Transform,
    font: Handle<Font>,
    tile_loader: TileLoader,
}

impl SpawnerData {
    pub fn new(
        materials: &mut Assets<StandardMaterial>,
        meshes: &mut Assets<Mesh>,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        Self {
            character: Appearance::new(materials, asset_server.load("tiles/character.png")),
            glass: Appearance::new(materials, Color::rgba(0.8, 0.9, 1.0, 0.2)), // transparant blue
            wood: Appearance::new(materials, Color::rgb(0.7, 0.6, 0.5)),
            whitish: Appearance::new(materials, Color::rgb(0.95, 0.93, 0.88)),
            wooden_wall: Appearance::new(materials, asset_server.load("tiles/wall.png")),
            yellow: Appearance::new(materials, Color::rgb(0.8, 0.8, 0.4)),
            tile_materials: HashMap::new(),
            character_mesh: meshes.add(Mesh::from(shape::Quad {
                size: Vec2::new(1.0, 1.0),
                flip: false,
            })),
            cube_mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            tile_meshes: HashMap::new(),
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
            font: asset_server.load("fonts/FiraMono-Medium.otf"),
            tile_loader: TileLoader::new(asset_server),
        }
    }
}

#[derive(SystemParam)]
pub struct Spawner<'w, 's> {
    commands: Commands<'w, 's>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    asset_server: Res<'w, AssetServer>,
    data: ResMut<'w, SpawnerData>,
}

impl<'w, 's> Spawner<'w, 's> {
    // Based on bevy_render-0.7.0/src/mesh/shape/mod.rs - line 194-223
    fn get_tile_mesh(&mut self, mesh_info: MeshInfo) -> Handle<Mesh> {
        self.data
            .tile_meshes
            .entry(mesh_info)
            .or_insert_with(|| {
                let extent = 1.0 / 2.0;

                let index = mesh_info.index as f32;
                let size = mesh_info.size as f32;
                let width = mesh_info.width as f32; // tiles per row
                let height = (size / width).ceil() as f32; // tiles per column

                let x_min = (index % width) / width;
                let x_max = x_min + 1.0 / width;
                let y_min = (index / width).floor() / height;
                let y_max = y_min + 1.0 / height;

                let vertices = [
                    ([extent, 0.0, -extent], [x_max, y_max]),
                    ([extent, 0.0, extent], [x_max, y_min]),
                    ([-extent, 0.0, extent], [x_min, y_min]),
                    ([-extent, 0.0, -extent], [x_min, y_max]),
                ];

                let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2]);
                let mut positions = Vec::new();
                let mut uvs = Vec::new();
                for (position, uv) in &vertices {
                    positions.push(*position);
                    uvs.push(*uv);
                }
                let normals = vec![[0.0, 1.0, 0.0]; 4];

                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_indices(Some(indices));
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

                self.meshes.add(mesh)
            })
            .clone()
    }

    fn get_tile_material(
        &mut self,
        imagepath: &str,
        alpha_mode: AlphaMode,
    ) -> Handle<StandardMaterial> {
        self.data
            .tile_materials
            .entry(imagepath.to_string())
            .or_insert_with(|| {
                self.materials.add(StandardMaterial {
                    base_color_texture: Some(self.asset_server.load(imagepath)),
                    alpha_mode,
                    ..StandardMaterial::default()
                })
            })
            .clone()
    }

    fn get_pbr_bundles(
        &mut self,
        tile_name: &TileName,
        sprite_infos: &Vec<SpriteInfo>,
        tile_type: TileType,
    ) -> Vec<PbrBundle> {
        let mut pbr_bundles = Vec::new();
        for sprite_info in sprite_infos {
            let scale = Vec3::new(
                sprite_info.scale.0,
                /*thickness*/ 1.0,
                sprite_info.scale.1,
            );
            let alpha_mode = if tile_type == TileType::Terrain
                && sprite_info.orientation == SpriteOrientation::Horizontal
                && (tile_name.0.starts_with("t_grass") || tile_name.0.starts_with("t_dirt"))
            {
                AlphaMode::Opaque
            } else {
                AlphaMode::Blend
            };
            pbr_bundles.push(PbrBundle {
                mesh: self.get_tile_mesh(sprite_info.mesh_info),
                material: self.get_tile_material(&sprite_info.imagepath, alpha_mode),
                transform: match sprite_info.orientation {
                    SpriteOrientation::Horizontal => Transform {
                        translation: Vec3::new(
                            /*back*/ sprite_info.offset.1,
                            /*up*/
                            match sprite_info.layer {
                                SpriteLayer::Front => 0.01,
                                _ => 0.0,
                            },
                            /*right*/ sprite_info.offset.0,
                        ),
                        rotation: Quat::from_rotation_y(0.5 * std::f32::consts::PI),
                        scale,
                    },
                    SpriteOrientation::Vertical | SpriteOrientation::Cube => Transform {
                        translation: Vec3::new(
                            /*back*/
                            match sprite_info.layer {
                                SpriteLayer::Front => -0.01,
                                _ => 0.0,
                            },
                            /*up*/
                            0.52 + sprite_info.offset.1,
                            /*right*/
                            sprite_info.offset.0,
                        ),
                        rotation: Quat::from_rotation_z(0.5 * std::f32::consts::PI)
                            * Quat::from_rotation_y(0.5 * std::f32::consts::PI),
                        scale,
                    },
                },
                ..PbrBundle::default()
            });
        }
        pbr_bundles
    }

    fn spawn_tile(&mut self, parent: Entity, pos: Pos, tile_name: &TileName, tile_type: TileType) {
        let label = tile_name.to_label();
        let tile_infos = self.data.tile_loader.sprite_infos(tile_name);
        let pbr_bundles = self.get_pbr_bundles(tile_name, &tile_infos, tile_type);

        self.commands.entity(parent).with_children(|child_builder| {
            let tile = child_builder
                .spawn_bundle((
                    pos,
                    label,
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
            } else if tile_infos
                .iter()
                .any(|t| t.orientation == SpriteOrientation::Cube)
            {
                child_builder.add_command(Insert {
                    entity: tile,
                    component: Obstacle,
                });
                child_builder.add_command(Insert {
                    entity: tile,
                    component: Opaque,
                });
            } else if tile_type == TileType::Terrain {
                child_builder.add_command(Insert {
                    entity: tile,
                    component: Floor,
                });
            }

            if tile_infos
                .iter()
                .any(|t| t.orientation == SpriteOrientation::Vertical)
            {
                if tile_name.0.starts_with("t_stairs_up")
                    || tile_name.0.starts_with("t_wood_stairs_up")
                    || tile_name.0.starts_with("t_ladder_up")
                    || tile_name.0.starts_with("t_ramp_up")
                    || tile_name.0.starts_with("t_gutter_downspout")
                {
                    child_builder.add_command(Insert {
                        entity: tile,
                        component: Stairs,
                    });
                } else {
                    match tile_infos
                        .iter()
                        .map(|t| (10.0 * t.scale.1) as i8)
                        .max()
                        .unwrap()
                    {
                        x if 25 < x => {
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
                        x if 15 < x => {
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
                                component: Hurdle(2.0),
                            });
                        }
                    }
                }
            }
        });
    }

    pub fn spawn_stairs_down(&mut self, parent: Entity, pos: Pos) {
        self.spawn_tile(
            parent,
            pos,
            &TileName::new("t_wood_stairs_down"),
            TileType::Terrain,
        );
    }

    pub fn spawn_roofing(&mut self, parent: Entity, pos: Pos) {
        self.spawn_tile(
            parent,
            pos,
            &TileName::new("t_shingle_flat_roof"),
            TileType::Terrain,
        );
    }

    pub fn spawn_wall(&mut self, pos: Pos) {
        self.commands
            .spawn_bundle((
                Wall,
                pos,
                Integrity::new(1000),
                Obstacle,
                Opaque,
                self.data.wooden_wall.clone(),
                Label::new("wall"),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.data.cube_mesh.clone(),
                transform: self.data.wall_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_window(&mut self, pos: Pos) {
        self.commands
            .spawn_bundle((
                Window,
                pos,
                Integrity::new(1),
                Obstacle,
                Hurdle(2.5),
                Label::new("window"),
                self.data.wooden_wall.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.data.cube_mesh.clone(),
                ..PbrBundle::default()
            })
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(PbrBundle {
                        mesh: self.data.cube_mesh.clone(),
                        transform: self.data.window_pane_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.data.glass.clone())
                    .insert(WindowPane);
            });
    }

    pub fn spawn_stairs(&mut self, pos: Pos) {
        self.commands
            .spawn_bundle((
                Stairs,
                pos,
                Integrity::new(100),
                Hurdle(1.5),
                Label::new("stairs"),
                self.data.wood.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.data.cube_mesh.clone(),
                transform: self.data.stair_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_rack(&mut self, pos: Pos) {
        self.commands
            .spawn_bundle((
                Rack,
                pos,
                Integrity::new(40),
                Obstacle,
                Opaque,
                Label::new("rack"),
                self.data.wood.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.data.cube_mesh.clone(),
                transform: self.data.rack_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_table(&mut self, pos: Pos) {
        self.commands
            .spawn_bundle((
                Table,
                pos,
                Integrity::new(30),
                Hurdle(2.0),
                Label::new("table"),
                self.data.wood.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.data.cube_mesh.clone(),
                transform: self.data.table_transform,
                ..PbrBundle::default()
            });
    }

    pub fn spawn_chair(&mut self, pos: Pos) {
        self.commands
            .spawn_bundle((
                Chair,
                pos,
                Integrity::new(10),
                Hurdle(1.5),
                Label::new("chair"),
                self.data.whitish.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.data.cube_mesh.clone(),
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
                        mesh: self.data.cube_mesh.clone(),
                        transform: Transform {
                            translation: Vec3::new(0.0, 0.2 / 0.45, -0.23 / 0.45),
                            rotation: Quat::from_rotation_y(-0.5 * std::f32::consts::PI),
                            scale: Vec3::new(0.05 / 0.45, 0.85 / 0.45, 1.0),
                        },
                        ..PbrBundle::default()
                    })
                    .insert(self.data.whitish.clone());
            });
    }

    pub fn spawn_containable(&mut self, label: Label, pos: Pos, containable: Containable) {
        let size = f32::from(containable.0).min(64.0).powf(0.33) / 4.0;

        self.commands
            .spawn_bundle((
                label,
                pos,
                containable,
                self.data.yellow.clone(),
                PlayerVisible::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.data.cube_mesh.clone(),
                // customizing the size using a transform allows better positioning
                transform: Transform::from_scale(Vec3::new(size, size, size)),
                ..PbrBundle::default()
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
        let focus = Vec3::new(1.0, 0.0, 0.1);
        let transform = Transform::from_scale(Vec3::new(1.0, 1.8, 1.0)).looking_at(focus, Vec3::Y);

        let entity = self
            .commands
            .spawn_bundle((label, pos, health, speed, faction, Obstacle, Container(4)))
            .insert_bundle((self.data.character.clone(), PlayerVisible::Reevaluate))
            /* TODO better as sub-entity?
            .with_children(|child_builder| {
                child_builder.spawn_bundle(PbrBundle {
                    mesh: self.character_mesh.clone(),
                    transform,
                    ..PbrBundle::default()
                    });
                })*/
            .insert_bundle(PbrBundle {
                mesh: self.data.character_mesh.clone(),
                transform,
                ..PbrBundle::default()
            })
            .id();

        if let Some(player) = player {
            let cursor_sprite_info = &self
                .data
                .tile_loader
                .sprite_infos(&TileName("cursor".to_string()))[0];
            let cursor_mesh = self.get_tile_mesh(cursor_sprite_info.mesh_info);
            let cursor_material =
                self.get_tile_material(&cursor_sprite_info.imagepath, AlphaMode::Blend);
            self.commands
                .entity(entity)
                .insert(player)
                .insert(PosYChanged)
                .insert(ZoneChanged)
                .with_children(|child_builder| {
                    child_builder
                        .spawn_bundle(PbrBundle {
                            transform: Transform::from_matrix(transform.compute_matrix().inverse()),
                            ..PbrBundle::default()
                        })
                        .with_children(|child_builder| {
                            child_builder
                                .spawn_bundle(PbrBundle::default())
                                .insert(CameraBase)
                                .with_children(|child_builder| {
                                    child_builder
                                        .spawn_bundle(PbrBundle {
                                            mesh: cursor_mesh,
                                            material: cursor_material,
                                            transform: Transform {
                                                translation: Vec3::new(
                                                    0.0,
                                                    0.15 - 0.5 * transform.scale.y,
                                                    0.0,
                                                ),
                                                scale: Vec3::new(1.15, 1.0, 1.15),
                                                ..Transform::default()
                                            },
                                            ..PbrBundle::default()
                                        })
                                        .insert(CameraCursor);

                                    child_builder
                                        .spawn_bundle(PbrBundle {
                                            transform: Transform::identity()
                                                .looking_at(focus, Vec3::Y),
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
                });
        }
    }

    pub fn spawn_gui(&mut self) {
        self.commands.spawn_bundle(UiCameraBundle::default());

        let text_style = TextStyle {
            font: self.data.font.clone(),
            font_size: 16.0,
            color: Color::rgb(0.8, 0.8, 0.8),
        };

        self.commands
            .spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "".to_string(),
                        style: text_style.clone(),
                    }],
                    ..Text::default()
                },
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(5.0),
                        left: Val::Px(5.0),
                        ..Rect::default()
                    },
                    ..Style::default()
                },
                ..TextBundle::default()
            })
            .insert(LogDisplay);

        self.commands
            .spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "\n".to_string(),
                            style: text_style.clone(),
                        };
                        6
                    ],
                    ..Text::default()
                },
                style: Style {
                    align_self: AlignSelf::FlexStart,
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(5.0),
                        right: Val::Px(5.0),
                        ..Rect::default()
                    },
                    ..Style::default()
                },
                ..TextBundle::default()
            })
            .insert(StatusDisplay);

        self.commands.spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "move      numpad\nup/down   r/f\npick/drop b/v\nattack    a\nrun       +\nexamine   x\nzoom      scroll wheel\nquit      ctrl+c/ctrl+d/ctrl+q".to_string(),
                        style: text_style,
                    },
                ],
                ..Text::default()
            },
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Rect::default()
                },
                ..Style::default()
            },
            ..TextBundle::default()
        }).insert(ManualDisplay);

        let light_transform = Transform::from_matrix(Mat4::from_euler(
            EulerRot::ZYX,
            0.0,
            -0.18 * std::f32::consts::TAU,
            -std::f32::consts::FRAC_PI_4,
        ));
        dbg!(&light_transform);
        self.commands.spawn_bundle(DirectionalLightBundle {
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
        let parent = self.commands.spawn_bundle(PbrBundle::default()).id();

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
