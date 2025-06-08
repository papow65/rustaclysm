use crate::SpriteLayer;

const SEPARATION_OFFSET: f32 = 0.005;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ObjectCategory {
    Terrain,
    Field,
    Furniture,
    Item,
    Character,
    Vehicle,
    VehiclePart,
    ZoneLevel,
    Meta,
}

impl ObjectCategory {
    #[must_use]
    pub const fn shading_applied(self) -> bool {
        !matches!(self, Self::ZoneLevel | Self::Meta)
    }

    #[must_use]
    pub(crate) const fn vertical_offset(self, layer: SpriteLayer) -> f32 {
        let level = 2 * match self {
            Self::ZoneLevel => -1,
            Self::Terrain => 0,
            Self::Field => 1,
            Self::Furniture => 2,
            Self::Item => 3,
            Self::Character => 5,
            Self::Vehicle => unimplemented!(),
            Self::VehiclePart => 4,
            Self::Meta => 6,
        } + match &layer {
            SpriteLayer::Front => 1,
            SpriteLayer::Back => 0,
        };

        level as f32 * SEPARATION_OFFSET
    }
}
