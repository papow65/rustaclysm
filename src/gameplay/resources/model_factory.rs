use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};
use std::path::PathBuf;

#[derive(Default, Resource)]
pub(crate) struct AppearanceCache(HashMap<PathBuf, Appearance>);

#[derive(Default, Resource)]
pub(crate) struct MeshCaches {
    horizontal_planes: HashMap<SpriteNumber, Handle<Mesh>>,
    vertical_planes: HashMap<SpriteNumber, Handle<Mesh>>,
    cuboids: HashMap<SpriteNumber, Handle<Mesh>>,
}

#[derive(SystemParam)]
pub(crate) struct ModelFactory<'w> {
    appearance_cache: ResMut<'w, AppearanceCache>,
    mesh_caches: ResMut<'w, MeshCaches>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    asset_server: Res<'w, AssetServer>,
}

impl<'w> ModelFactory<'w> {
    fn get_mesh(&mut self, model: &Model) -> Handle<Mesh> {
        match model.shape {
            ModelShape::Plane {
                orientation: SpriteOrientation::Horizontal,
                ..
            } => &mut self.mesh_caches.horizontal_planes,
            ModelShape::Plane {
                orientation: SpriteOrientation::Vertical,
                ..
            } => &mut self.mesh_caches.vertical_planes,
            ModelShape::Cuboid { .. } => &mut self.mesh_caches.cuboids,
        }
        .entry(model.sprite_number)
        .or_insert_with(|| self.mesh_assets.add(model.to_mesh()))
        .clone()
    }

    pub(crate) fn get_appearance(&mut self, model: &Model) -> Appearance {
        self.appearance_cache
            .0
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

    pub(crate) fn get_pbr_bundle(&mut self, model: &Model, shaded: bool) -> PbrBundle {
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
}