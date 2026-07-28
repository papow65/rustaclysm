use gameplay_cdda_common::ObjectCategory;

const SEPARATION_OFFSET: f32 = 0.005;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SpriteLayer {
    Front,
    Back,
}

impl SpriteLayer {
    #[must_use]
    pub(crate) const fn vertical_offset(self, category: ObjectCategory) -> f32 {
        let level = 2 * match category {
            ObjectCategory::ZoneLevel => -1,
            ObjectCategory::Terrain => 0,
            ObjectCategory::Field => 1,
            ObjectCategory::Furniture => 2,
            ObjectCategory::Item => 3,
            ObjectCategory::Character => 5,
            ObjectCategory::Vehicle => unimplemented!(),
            ObjectCategory::VehiclePart => 4,
            ObjectCategory::Meta => 6,
        } + match &self {
            Self::Front => 1,
            Self::Back => 0,
        };

        level as f32 * SEPARATION_OFFSET
    }
}
