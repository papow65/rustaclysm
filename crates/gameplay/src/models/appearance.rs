use crate::LastSeen;
use bevy::prelude::{AlphaMode, Assets, Color, Component, MeshMaterial3d, Srgba, StandardMaterial};

#[derive(Clone, Debug, Component)]
#[component(immutable)]
pub(crate) struct Appearance {
    seen: MeshMaterial3d<StandardMaterial>,
    remembered: MeshMaterial3d<StandardMaterial>,
}

impl Appearance {
    pub(super) fn new<T>(materials: &mut Assets<StandardMaterial>, material: T) -> Self
    where
        T: Into<StandardMaterial>,
    {
        let mut material = material.into();
        material.alpha_mode = AlphaMode::Blend;
        let remembered = materials.add(StandardMaterial {
            base_color_texture: material.base_color_texture.clone(),
            base_color: Self::remembered(material.base_color),
            alpha_mode: AlphaMode::Blend,
            ..StandardMaterial::default()
        });
        Self {
            seen: materials.add(material).into(),
            remembered: remembered.into(),
        }
    }

    pub(crate) fn fixed_material(&self) -> MeshMaterial3d<StandardMaterial> {
        self.material(&LastSeen::Currently)
    }

    pub(crate) fn material(&self, last_seen: &LastSeen) -> MeshMaterial3d<StandardMaterial> {
        match last_seen {
            LastSeen::Currently => self.seen.clone(),
            LastSeen::Previously => self.remembered.clone(),
            LastSeen::Never => panic!("material(...) called when never seen"),
        }
    }

    fn remembered(color: Color) -> Color {
        let srgba = Srgba::from(color);
        Color::srgba(
            srgba.red * 0.6,
            srgba.green * 0.6,
            srgba.blue,
            0.5_f32.mul_add(srgba.alpha, 0.5),
        )
    }
}
