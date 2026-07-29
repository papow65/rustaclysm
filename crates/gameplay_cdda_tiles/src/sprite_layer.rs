use cdda_json_files::DynInfoId;

const SEPARATION_OFFSET: f32 = 0.005;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SpriteLayer {
    Front,
    Back,
}

impl SpriteLayer {
    #[must_use]
    pub(crate) fn vertical_offset(self, dyn_info_id: &DynInfoId) -> f32 {
        let level = 2 * match dyn_info_id {
            DynInfoId::OvermapTerrain(..) => -1,
            DynInfoId::Terrain(..) => 0,
            DynInfoId::Field(..) => 1,
            DynInfoId::Furniture(..) => 2,
            DynInfoId::Item(..) => 3,
            DynInfoId::Character(..) => 5,
            DynInfoId::VehiclePart(..) => 4,
            DynInfoId::Untyped(..) => 6,
            other_dyn_info_id => unimplemented!("{other_dyn_info_id:?}"),
        } + match &self {
            Self::Front => 1,
            Self::Back => 0,
        };

        level as f32 * SEPARATION_OFFSET
    }
}
