use crate::gameplay::{
    Appearance, Infos, LastSeen, Layers, Model, ModelShape, ObjectDefinition, SpriteOrientation,
    TileLoader, TileVariant,
};
use bevy::prelude::{
    AssetServer, Assets, Mesh, Mesh3d, MeshMaterial3d, Res, ResMut, Resource, StandardMaterial,
    Transform, Visibility,
};
use bevy::{ecs::system::SystemParam, utils::HashMap};
use cdda_json_files::SpriteNumber;
use std::path::PathBuf;

#[derive(Default, Resource)]
pub(crate) struct AppearanceCache(HashMap<PathBuf, Appearance>);

#[derive(Default, Resource)]
pub(crate) struct MeshCaches {
    horizontal_planes: HashMap<SpriteNumber, Mesh3d>,
    vertical_planes: HashMap<SpriteNumber, Mesh3d>,
    cuboids: HashMap<SpriteNumber, Mesh3d>,
}

#[derive(SystemParam)]
pub(crate) struct ModelFactory<'w> {
    appearance_cache: ResMut<'w, AppearanceCache>,
    mesh_caches: ResMut<'w, MeshCaches>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    asset_server: Res<'w, AssetServer>,
    infos: Res<'w, Infos>,
    loader: Res<'w, TileLoader>,
}

impl<'w> ModelFactory<'w> {
    fn get_mesh(&mut self, model: &Model) -> Mesh3d {
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
        .or_insert_with(|| self.mesh_assets.add(model.to_mesh()).into())
        .clone()
    }

    fn get_appearance(&mut self, model: &Model) -> Appearance {
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

    fn get_pbr_bundle(
        &mut self,
        model: &Model,
        visibility: Visibility,
        shaded: bool,
    ) -> (
        Mesh3d,
        MeshMaterial3d<StandardMaterial>,
        Transform,
        Visibility,
    ) {
        (
            self.get_mesh(model),
            if shaded {
                MeshMaterial3d::<StandardMaterial>::default()
            } else {
                self.get_appearance(model).material(&LastSeen::Currently)
            },
            model.to_transform(),
            visibility,
        )
    }

    pub(crate) fn get_single_pbr_bundle(
        &mut self,
        definition: &ObjectDefinition,
    ) -> (
        Mesh3d,
        MeshMaterial3d<StandardMaterial>,
        Transform,
        Visibility,
    ) {
        let models = self
            .loader
            .get_models(definition, &self.infos.variants(definition), None);
        assert!(models.overlay.is_none(), "{models:?}");
        self.get_pbr_bundle(&models.base, Visibility::Hidden, false)
    }

    pub(crate) fn get_layers(
        &mut self,
        definition: &ObjectDefinition,
        visibility: Visibility,
        shading: bool,
        tile_variant: Option<TileVariant>,
    ) -> Layers<(
        (
            Mesh3d,
            MeshMaterial3d<StandardMaterial>,
            Transform,
            Visibility,
        ),
        Appearance,
    )> {
        let models =
            self.loader
                .get_models(definition, &self.infos.variants(definition), tile_variant);
        models.map_mut(|model| {
            (
                self.get_pbr_bundle(&model, visibility, shading),
                self.get_appearance(&model),
            )
        })
    }
}
