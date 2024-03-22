use serde::Deserialize;

#[derive(Clone, Hash, PartialEq, Eq, Debug, Deserialize)]
pub(crate) struct TypeId(&'static str);

impl TypeId {
    pub(crate) const CHARACTER: &'static [Self] = &[Self("MONSTER")];

    pub(crate) const ITEM: &'static [Self] = &[
        Self("AMMO"),
        Self("ARMOR"),
        Self("BATTERY"),
        Self("BIONIC_ITEM"),
        Self("BOOK"),
        Self("COMESTIBLE"),
        Self("ENGINE"),
        Self("GENERIC"),
        Self("GUN"),
        Self("GUNMOD"),
        Self("MAGAZINE"),
        Self("PET_ARMOR"),
        Self("SPELL"),
        Self("TOOL"),
        Self("TOOL_ARMOR"),
        Self("TOOLMOD"),
        Self("WHEEL"),
    ];

    pub(crate) const FIELD: &'static [Self] = &[Self("field_type")];
    pub(crate) const FURNITURE: &'static [Self] = &[Self("furniture")];
    pub(crate) const ITEM_GROUP: &'static [Self] = &[Self("item_group")];

    pub(crate) const OVERMAP: &'static [Self] = &[
        Self("city_building"),
        Self("map_extra"),
        Self("overmap_connection"),
        Self("overmap_land_use_code"),
        Self("overmap_location"),
        Self("overmap_special"),
        Self("overmap_special_migration"),
    ];

    pub(crate) const TERRAIN: &'static [Self] = &[Self("terrain")];
    pub(crate) const TOOL_QUALITY: &'static [Self] = &[Self("tool_quality")];
    pub(crate) const VEHICLE_PART: &'static [Self] = &[Self("vehicle_part")];

    pub(crate) const MIGRATION: &'static [Self] = &[
        Self("MIGRATION"),
        Self("overmap_terrain"),
        Self("vehicle_part_migration"),
    ];

    // TODO use these types
    pub(crate) const UNUSED: &'static [Self] = &[
        Self("ammunition_type"),
        Self("behavior"),
        Self("enchantment"),
        Self("effect_on_condition"),
        Self("effect_type"),
        Self("fault"),
        Self("json_flag"),
        Self("mapgen"),
        Self("monster_flag"),
        Self("monstergroup"),
        Self("nested_category"),
        Self("practice"),
        Self("recipe"),
        Self("recipe_category"),
        Self("recipe_group"),
        Self("requirement"),
        Self("ter_furn_transform"),
        Self("TRAIT_MIGRATION"),
        Self("trap"),
        Self("uncraft"),
        Self("vehicle_part_category"),
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
            Self::TOOL_QUALITY,
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
