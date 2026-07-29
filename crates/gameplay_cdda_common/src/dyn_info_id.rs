use cdda_json_files::{
    CddaVehicle, CharacterInfo, CommonItemInfo, FieldInfo, FurnitureInfo, InfoId,
    OvermapTerrainInfo, TerrainInfo, UntypedInfoId, VehiclePartInfo,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DynInfoId {
    Terrain(InfoId<TerrainInfo>),
    Field(InfoId<FieldInfo>),
    Furniture(InfoId<FurnitureInfo>),
    Item(InfoId<CommonItemInfo>),
    Character(InfoId<CharacterInfo>),
    Vehicle(InfoId<CddaVehicle>),
    VehiclePart(InfoId<VehiclePartInfo>),
    ZoneLevel(InfoId<OvermapTerrainInfo>),
    Meta(UntypedInfoId),
}

impl DynInfoId {
    #[must_use]
    pub const fn shading_applied(&self) -> bool {
        !matches!(self, Self::ZoneLevel(..) | Self::Meta(..))
    }

    #[must_use]
    pub fn modified(&self, modify: impl Fn(&UntypedInfoId) -> UntypedInfoId) -> Self {
        match self {
            Self::Terrain(info_id) => Self::Terrain(InfoId::from(modify(info_id.untyped()))),
            Self::Field(info_id) => Self::Field(InfoId::from(modify(info_id.untyped()))),
            Self::Furniture(info_id) => Self::Furniture(InfoId::from(modify(info_id.untyped()))),
            Self::Item(info_id) => Self::Item(InfoId::from(modify(info_id.untyped()))),
            Self::Character(info_id) => Self::Character(InfoId::from(modify(info_id.untyped()))),
            Self::Vehicle(info_id) => Self::Vehicle(InfoId::from(modify(info_id.untyped()))),
            Self::VehiclePart(info_id) => {
                Self::VehiclePart(InfoId::from(modify(info_id.untyped())))
            }
            Self::ZoneLevel(info_id) => Self::ZoneLevel(InfoId::from(modify(info_id.untyped()))),
            Self::Meta(untyped_info_id) => Self::Meta(modify(untyped_info_id)),
        }
    }

    #[must_use]
    pub const fn untyped(&self) -> &UntypedInfoId {
        match self {
            Self::Terrain(info_id) => info_id.untyped(),
            Self::Field(info_id) => info_id.untyped(),
            Self::Furniture(info_id) => info_id.untyped(),
            Self::Item(info_id) => info_id.untyped(),
            Self::Character(info_id) => info_id.untyped(),
            Self::Vehicle(info_id) => info_id.untyped(),
            Self::VehiclePart(info_id) => info_id.untyped(),
            Self::ZoneLevel(info_id) => info_id.untyped(),
            Self::Meta(info_id) => info_id,
        }
    }
}
