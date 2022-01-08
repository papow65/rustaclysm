use bevy::math::Quat;
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;

use super::super::components::Window;
use super::super::components::*;
use super::super::units::*;
use super::tile_loader::*;
use super::zone_loader::*;

pub struct Spawner<'a> {
    commands: Commands<'a>,
    character: Appearance,
    blackish: Appearance,
    glass: Appearance,
    wood: Appearance,
    whitish: Appearance,
    wooden_wall: Appearance,
    yellow: Appearance,
    character_mesh: Handle<Mesh>,
    cube_mesh: Handle<Mesh>,
    wall_transform: Transform,
    window_pane_transform: Transform,
    stair_transform: Transform,
    rack_transform: Transform,
    table_transform: Transform,
    font: Handle<Font>,
    tile_loader: TileLoader,
}

impl<'a> Spawner<'a> {
    pub fn new(
        commands: Commands<'a>,
        materials: &mut Assets<StandardMaterial>,
        meshes: &mut Assets<Mesh>,
        texture_atlases: &mut Assets<TextureAtlas>,
        asset_server: &Res<AssetServer>,
    ) -> Spawner<'a> {
        Spawner {
            commands,
            character: Appearance::new(materials, asset_server.load("tiles/character.png")),
            blackish: Appearance::new(materials, Color::rgb(0.15, 0.15, 0.15)),
            glass: Appearance::new(materials, Color::rgba(0.8, 0.9, 1.0, 0.2)), // transparant blue
            wood: Appearance::new(materials, Color::rgb(0.7, 0.6, 0.5)),
            whitish: Appearance::new(materials, Color::rgb(0.95, 0.93, 0.88)),
            wooden_wall: Appearance::new(materials, asset_server.load("tiles/wall.png")),
            yellow: Appearance::new(materials, Color::rgb(0.8, 0.8, 0.4)),
            character_mesh: meshes.add(Mesh::from(shape::Quad {
                size: Vec2::new(1.0, 1.0),
                flip: false,
            })),
            cube_mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
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
            tile_loader: TileLoader::new(texture_atlases, asset_server),
        }
    }

    fn spawn_tile(&mut self, pos: Pos, tile_name: &TileName) -> Entity {
        let rotation = Quat::from_rotation_z(0.01 * std::f32::consts::PI)
            * Quat::from_rotation_y(1.5 * std::f32::consts::PI)
            * Quat::from_rotation_x(1.5 * std::f32::consts::PI);
        let label = tile_name.to_label();
        println!("{:?}", tile_name);
        self.commands
            .spawn_bundle((
                Floor,
                pos,
                label,
                Transform::from_translation(pos.vec3(0.0)),
                GlobalTransform::default(),
                Visible::default(),
            ))
            .with_children(|child_builder| {
                /*if pos.1 == 0 {
                    for sprite in self.tile_loader.debug(pos, &rotation) {
                        child_builder.spawn_bundle(sprite);
                    }
                }*/
                for sprite in self.tile_loader.sprite_sheet_bundles(tile_name, &rotation) {
                    child_builder.spawn_bundle(sprite);
                }
            })
            .id()
    }

    pub fn spawn_stairs_down(&mut self, pos: Pos) {
        let entity = self.spawn_tile(pos, &TileName::new("t_wood_stairs_down"));
        self.commands.entity(entity).insert(StairsDown);
    }

    pub fn spawn_grass(&mut self, pos: Pos) {
        self.spawn_tile(pos, &TileName::new("t_grass"));
    }

    pub fn spawn_stone_floor(&mut self, pos: Pos) {
        self.spawn_tile(pos, &TileName::new("t_concrete"));
    }

    pub fn spawn_roofing(&mut self, pos: Pos) {
        self.spawn_tile(pos, &TileName::new("t_shingle_flat_roof"));
    }

    pub fn spawn_wall(&mut self, pos: Pos) {
        self.commands
            .spawn_bundle((
                Wall,
                pos,
                Integrity::new(1000),
                Obstacle,
                Opaque,
                self.wooden_wall.clone(),
                Label::new("wall"),
                Visibility::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
                transform: self.wall_transform,
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
                self.wooden_wall.clone(),
                Visibility::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
                ..PbrBundle::default()
            })
            .with_children(|child_builder| {
                child_builder
                    .spawn_bundle(PbrBundle {
                        mesh: self.cube_mesh.clone(),
                        visible: Visible {
                            is_transparent: true,
                            ..Visible::default()
                        },
                        transform: self.window_pane_transform,
                        ..PbrBundle::default()
                    })
                    .insert(self.glass.clone())
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
                self.wood.clone(),
                Visibility::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
                transform: self.stair_transform,
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
                self.wood.clone(),
                Visibility::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
                transform: self.rack_transform,
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
                self.wood.clone(),
                Visibility::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
                transform: self.table_transform,
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
                self.whitish.clone(),
                Visibility::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
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
                        mesh: self.cube_mesh.clone(),
                        transform: Transform {
                            translation: Vec3::new(0.0, 0.2 / 0.45, -0.23 / 0.45),
                            rotation: Quat::from_rotation_y(-0.5 * std::f32::consts::PI),
                            scale: Vec3::new(0.05 / 0.45, 0.85 / 0.45, 1.0),
                        },
                        ..PbrBundle::default()
                    })
                    .insert(self.whitish.clone());
            });
    }

    pub fn spawn_containable(&mut self, label: Label, pos: Pos, containable: Containable) {
        let size = (containable.0 as f32).min(64.0).powf(0.33) / 4.0;

        self.commands
            .spawn_bundle((
                label,
                pos,
                containable,
                self.yellow.clone(),
                Visibility::Reevaluate,
            ))
            .insert_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
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
        let transform = Transform::from_scale(Vec3::new(1.0, 1.8, 1.0))
            .looking_at(Vec3::new(1.0, 0.0, 0.1), Vec3::Y);

        let entity = self
            .commands
            .spawn_bundle((label, pos, health, speed, faction, Obstacle, Container(4)))
            .insert_bundle((self.character.clone(), Visibility::Reevaluate))
            .insert_bundle(PbrBundle {
                mesh: self.character_mesh.clone(),
                transform,
                visible: Visible {
                    is_transparent: true,
                    ..Visible::default()
                },
                ..PbrBundle::default()
            })
            .id();

        if let Some(player) = player {
            self.commands
                .entity(entity)
                .insert(player)
                .insert(PosYChanged)
                .with_children(|child_builder| {
                    // ChildBuilder has a similar API to Commands
                    child_builder
                        .spawn_bundle(PbrBundle {
                            transform: Transform::from_scale(Vec3::new(
                                1.0 / transform.scale.x,
                                1.0 / transform.scale.y,
                                1.0 / transform.scale.z,
                            )),
                            ..PbrBundle::default()
                        })
                        .with_children(|child_builder| {
                            child_builder.spawn_bundle(PerspectiveCameraBundle {
                                perspective_projection: PerspectiveProjection {
                                    fov: 0.3,
                                    ..PerspectiveProjection::default()
                                },
                                ..PerspectiveCameraBundle::default()
                            });
                        });
                });
        }
    }

    pub fn spawn_grid_lines(&mut self) {
        for x in 0..=SIZE.0 {
            self.commands.spawn_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
                material: self.blackish.material(Visibility::Seen),
                transform: Transform {
                    translation: Vec3::new(x as f32 - 0.5, 0.0, 0.5 * SIZE.2 as f32 - 0.5),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(0.01, 0.01, SIZE.2 as f32),
                },
                ..PbrBundle::default()
            });
        }

        for z in 0..=SIZE.2 {
            self.commands.spawn_bundle(PbrBundle {
                mesh: self.cube_mesh.clone(),
                material: self.blackish.material(Visibility::Seen),
                transform: Transform {
                    translation: Vec3::new(0.5 * SIZE.0 as f32 - 0.5, 0.0, z as f32 - 0.5),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(SIZE.0 as f32, 0.01, 0.01),
                },
                ..PbrBundle::default()
            });
        }
    }

    pub fn spawn_gui(&mut self) {
        self.commands.spawn_bundle(UiCameraBundle::default());

        let text_style = TextStyle {
            font: self.font.clone(),
            font_size: 20.0,
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
                    sections: vec!["    fps: ", "", "\n  time: ", "", "\nhealth: ", ""]
                        .iter()
                        .map(|s| TextSection {
                            value: s.to_string(),
                            style: text_style.clone(),
                        })
                        .collect::<Vec<TextSection>>(),
                    ..Text::default()
                },
                style: Style {
                    align_self: AlignSelf::FlexEnd,
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
                        value: "move      numpad\nup/down   r/f\npick/drop b/v\nattack    a\nstatus    ,\nzoom      scroll wheel\nquit      esc/ctrl+c/ctrl+d".to_string(),
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

        self.commands.spawn_bundle(LightBundle {
            light: Light {
                range: 10_000_000.0,
                intensity: 4_000_000.0,
                ..Light::default()
            },
            transform: Transform::from_xyz(
                -600.0 * f32::cos(50.0 / 180.0 * std::f32::consts::PI),
                1000.0 * f32::sin(50.0 / 180.0 * std::f32::consts::PI),
                -400.0 * f32::cos(50.0 / 180.0 * std::f32::consts::PI),
            ),
            ..LightBundle::default()
        });
    }

    pub fn spawn_floors(&mut self) {
        for y in 0..=2 {
            let x_range = match y {
                0 => 0..SIZE.0,
                _ => 17..24,
            };
            for x in x_range {
                let z_range = match y {
                    0 => 0..SIZE.0,
                    1 => 17..26,
                    _ => 21..26,
                };
                for z in z_range {
                    let pos = Pos(x, y, z);
                    match pos {
                        Pos(18, 1, 24) | Pos(18, 2, 22) => {
                            self.spawn_stairs_down(pos);
                        }
                        Pos(x, 1, z) if (18..=22).contains(&x) && (18..=24).contains(&z) => {
                            self.spawn_stone_floor(pos);
                        }
                        Pos(x, 0, z) if (16..=23).contains(&x) && (17..=26).contains(&z) => {
                            self.spawn_stone_floor(pos);
                        }
                        Pos(x, 0, z) if 48 <= x || 48 <= z => {
                            continue;
                        }
                        Pos(_, 0, _) => {
                            self.spawn_grass(pos);
                        }
                        _ => {
                            self.spawn_roofing(pos);
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
            //Pos(12, 0, 13),
            Pos(45, 0, 45),
            Health::new(10),
            Speed::from_h_kmph(6),
            Faction::Human,
            Some(Player {
                camera_distance: 3.0,
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

    pub fn load_cdda_region(&mut self, zone_pos: Pos, zones: Pos, from: Pos) {
        for x in 0..zones.0 {
            for y in 0..zones.1 {
                for z in 0..zones.2 {
                    if let Some(zone_layout) =
                        zone_layout(Pos(zone_pos.0 + x, zone_pos.1 + y, zone_pos.2 + z))
                    {
                        self.spawn_zone(
                            &zone_layout,
                            Pos(from.0 + 24 * x, from.1 + y, from.2 + 24 * z),
                        );
                    }
                }
            }
        }
    }

    fn spawn_zone(&mut self, zone_layout: &ZoneLayout, to: Pos) {
        let base = zone_layout.subzone_layouts[0].coordinates;
        for subzone_layout in &zone_layout.subzone_layouts {
            assert!(base.2 == subzone_layout.coordinates.2);
            self.spawn_subzone(
                subzone_layout,
                Pos(
                    to.0 + 12 * (subzone_layout.coordinates.0 - base.0),
                    to.1,
                    to.2 + 12 * (subzone_layout.coordinates.1 - base.1),
                ),
            );
        }
    }

    fn spawn_subzone(&mut self, subzone_layout: &SubzoneLayout, to: Pos) {
        for x in 0..12 {
            for z in 0..12 {
                let pos = Pos(to.0 + x, to.1, to.2 + z);
                let tile_name = &subzone_layout.terrain[(x + 12 * z) as usize];
                if tile_name != &TileName::new("t_open_air")
                    && tile_name != &TileName::new("t_open_air_rooved")
                {
                    self.spawn_tile(pos, tile_name);
                }

                for tile_name in subzone_layout
                    .furniture
                    .iter()
                    .filter_map(|at| at.get(Pos(x, 0, z)))
                {
                    self.spawn_tile(pos, tile_name);
                }

                if let Some(tile_name) = subzone_layout
                    .items
                    .iter()
                    .find_map(|at| at.get(Pos(x, 0, z)))
                    .map(|item| &item.typeid)
                {
                    self.spawn_tile(pos, tile_name);
                }
            }
        }
    }
}
