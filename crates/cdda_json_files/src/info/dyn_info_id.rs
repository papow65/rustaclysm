use crate::{
    CddaVehicle, CharacterInfo, CommonItemInfo, FieldInfo, FurnitureInfo, InfoId,
    OvermapTerrainInfo, TerrainInfo, UntypedInfoId, VehiclePartInfo,
};
use std::any::{Any, TypeId};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DynInfoId {
    Terrain(InfoId<TerrainInfo>),
    Field(InfoId<FieldInfo>),
    Furniture(InfoId<FurnitureInfo>),
    Item(InfoId<CommonItemInfo>),
    Character(InfoId<CharacterInfo>),
    Vehicle(InfoId<CddaVehicle>),
    VehiclePart(InfoId<VehiclePartInfo>),
    OvermapTerrain(InfoId<OvermapTerrainInfo>),
    Untyped(UntypedInfoId),
    WithTypeName {
        info_id: UntypedInfoId,
        type_: TypeId,
    },
}

impl DynInfoId {
    #[must_use]
    pub fn modified(&self, modify: impl Fn(&UntypedInfoId) -> UntypedInfoId) -> Self {
        let modified_id = modify(self.untyped());
        match self {
            Self::Terrain(_) => Self::Terrain(InfoId::from(modified_id)),
            Self::Field(_) => Self::Field(InfoId::from(modified_id)),
            Self::Furniture(_) => Self::Furniture(InfoId::from(modified_id)),
            Self::Item(_) => Self::Item(InfoId::from(modified_id)),
            Self::Character(_) => Self::Character(InfoId::from(modified_id)),
            Self::Vehicle(_) => Self::Vehicle(InfoId::from(modified_id)),
            Self::VehiclePart(_) => Self::VehiclePart(InfoId::from(modified_id)),
            Self::OvermapTerrain(_) => Self::OvermapTerrain(InfoId::from(modified_id)),
            Self::Untyped(_) => Self::Untyped(modified_id),
            Self::WithTypeName { .. } => unimplemented!(),
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
            Self::OvermapTerrain(info_id) => info_id.untyped(),
            Self::Untyped(info_id) | Self::WithTypeName { info_id, .. } => info_id,
        }
    }
}

impl<T> From<InfoId<T>> for DynInfoId
where
    T: 'static,
{
    fn from(info_id: InfoId<T>) -> Self {
        let boxed: Box<dyn Any> = Box::new(info_id);

        let boxed = match boxed.downcast::<InfoId<TerrainInfo>>() {
            Ok(info_id) => {
                return Self::Terrain(*info_id);
            }
            Err(boxed) => boxed,
        };

        let boxed = match boxed.downcast::<InfoId<FieldInfo>>() {
            Ok(info_id) => {
                return Self::Field(*info_id);
            }
            Err(boxed) => boxed,
        };

        let boxed = match boxed.downcast::<InfoId<FurnitureInfo>>() {
            Ok(info_id) => {
                return Self::Furniture(*info_id);
            }
            Err(boxed) => boxed,
        };

        let boxed = match boxed.downcast::<InfoId<CommonItemInfo>>() {
            Ok(info_id) => {
                return Self::Item(*info_id);
            }
            Err(boxed) => boxed,
        };

        let boxed = match boxed.downcast::<InfoId<CharacterInfo>>() {
            Ok(info_id) => {
                return Self::Character(*info_id);
            }
            Err(boxed) => boxed,
        };

        let boxed = match boxed.downcast::<InfoId<CddaVehicle>>() {
            Ok(info_id) => {
                return Self::Vehicle(*info_id);
            }
            Err(boxed) => boxed,
        };

        let boxed = match boxed.downcast::<InfoId<VehiclePartInfo>>() {
            Ok(info_id) => {
                return Self::VehiclePart(*info_id);
            }
            Err(boxed) => boxed,
        };

        let boxed = match boxed.downcast::<InfoId<OvermapTerrainInfo>>() {
            Ok(info_id) => {
                return Self::OvermapTerrain(*info_id);
            }
            Err(boxed) => boxed,
        };

        //let full_type_name = type_name::<T>();
        //let type_short = full_type_name.split("::").last().unwrap_or(full_type_name);
        Self::WithTypeName {
            info_id: boxed
                .downcast::<InfoId<T>>()
                .expect("Should be compatibel with its original type")
                .untyped()
                .clone(),
            type_: TypeId::of::<T>(),
        }
    }
}

impl From<UntypedInfoId> for DynInfoId {
    fn from(info_id: UntypedInfoId) -> Self {
        Self::Untyped(info_id)
    }
}
