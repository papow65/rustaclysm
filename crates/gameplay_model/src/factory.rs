use crate::{Appearance, AppearanceCache, MeshCaches};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    AssetServer, Assets, Mesh, Mesh3d, MeshMaterial3d, Res, ResMut, StandardMaterial, Transform,
    Vec3,
};
use cdda_json_files::UntypedInfoId;
use gameplay_cdda::{
    Infos, Layers, Model, ModelShape, ObjectCategory, SpriteOrientation, TileLoader, TileVariant,
};

#[derive(SystemParam)]
pub struct ModelFactory<'w> {
    appearance_cache: ResMut<'w, AppearanceCache>,
    mesh_caches: ResMut<'w, MeshCaches>,
    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    asset_server: Res<'w, AssetServer>,
    infos: Res<'w, Infos>,
    loader: Res<'w, TileLoader>,
}

impl ModelFactory<'_> {
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

    pub fn get_layers(
        &mut self,
        info_id: &UntypedInfoId,
        category: ObjectCategory,
        tile_variant: Option<TileVariant>,
    ) -> Layers<(Mesh3d, Transform, Appearance)> {
        let models = self.loader.get_models(
            info_id,
            category,
            &self.infos.variants(info_id.clone(), category),
            tile_variant,
        );
        models.map_mut(|model| {
            (
                self.get_mesh(&model),
                model.to_transform(),
                self.get_appearance(&model),
            )
        })
    }

    pub fn get_cursor(&mut self) -> (Mesh3d, Transform, MeshMaterial3d<StandardMaterial>) {
        let info_id = UntypedInfoId::new("cursor");
        let models = self.loader.get_models(
            &info_id,
            ObjectCategory::Meta,
            &self.infos.variants(info_id.clone(), ObjectCategory::Meta),
            None,
        );
        assert!(models.overlay.is_none(), "{models:?}");

        let mesh3d = self.get_mesh(&models.base);
        let mut transform = models.base.to_transform();
        transform.translation.y = 0.1;
        transform.scale = Vec3::new(1.1, 1.0, 1.1);

        (
            mesh3d,
            transform,
            self.get_appearance(&models.base).fixed_material(),
        )
    }
}
