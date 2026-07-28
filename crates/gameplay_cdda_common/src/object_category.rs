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
}
