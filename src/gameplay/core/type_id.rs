use serde::Deserialize;

#[derive(Clone, Hash, PartialEq, Eq, Debug, Deserialize)]
pub(crate) struct TypeId(&'static str);

impl TypeId {
    pub(crate) const CHARACTER: &[Self] = &[TypeId("MONSTER")];

    pub(crate) const ITEM: &[Self] = &[
        TypeId("AMMO"),
        TypeId("ARMOR"),
        TypeId("BATTERY"),
        TypeId("BIONIC_ITEM"),
        TypeId("BOOK"),
        TypeId("COMESTIBLE"),
        TypeId("ENGINE"),
        TypeId("GENERIC"),
        TypeId("GUN"),
        TypeId("GUNMOD"),
        TypeId("MAGAZINE"),
        TypeId("PET_ARMOR"),
        TypeId("SPELL"),
        TypeId("TOOL"),
        TypeId("TOOL_ARMOR"),
        TypeId("TOOLMOD"),
        TypeId("WHEEL"),
    ];

    pub(crate) const FIELD: &[Self] = &[TypeId("field_type")];
    pub(crate) const FURNITURE: &[Self] = &[TypeId("furniture")];
    pub(crate) const ITEM_GROUP: &[Self] = &[TypeId("item_group")];

    pub(crate) const OVERMAP: &[Self] = &[
        TypeId("city_building"),
        TypeId("map_extra"),
        TypeId("overmap_connection"),
        TypeId("overmap_land_use_code"),
        TypeId("overmap_location"),
        TypeId("overmap_special"),
        TypeId("overmap_special_migration"),
    ];

    pub(crate) const TERRAIN: &[Self] = &[TypeId("terrain")];
    pub(crate) const VEHICLE_PART: &[Self] = &[TypeId("vehicle_part")];

    pub(crate) const MIGRATION: &[Self] = &[
        TypeId("MIGRATION"),
        TypeId("overmap_terrain"),
        TypeId("vehicle_part_migration"),
    ];

    pub(crate) const UNUSED: &[Self] = &[
        TypeId("ammunition_type"),
        TypeId("behavior"),
        TypeId("enchantment"),
        TypeId("effect_on_condition"),
        TypeId("fault"),
        TypeId("json_flag"),
        TypeId("mapgen"),
        TypeId("TRAIT_MIGRATION"),
        TypeId("vehicle_part_category"),
    ];

    pub(crate) const fn all() -> &'static [&'static [Self]] {
        &[
            Self::CHARACTER,
            Self::FIELD,
            Self::FURNITURE,
            Self::ITEM,
            Self::ITEM_GROUP,
            Self::MIGRATION,
            Self::OVERMAP,
            Self::TERRAIN,
            Self::VEHICLE_PART,
            Self::UNUSED,
        ]
    }

    pub(crate) fn get(value: &'_ str) -> &Self {
        Self::all()
            .iter()
            .copied()
            .flatten()
            .find(|t| t.0 == value)
            .unwrap_or_else(|| panic!("{value} not found"))
    }
}
