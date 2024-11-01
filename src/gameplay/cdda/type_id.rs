use serde::Deserialize;

#[derive(Clone, Hash, PartialEq, Eq, Debug, Deserialize)]
pub(crate) struct TypeId(&'static str);

impl TypeId {
    pub(crate) const CHARACTER: &'static [Self] = &[Self("MONSTER")];

    // Item types
    pub(crate) const AMMO: &'static [Self] = &[Self("AMMO")];
    pub(crate) const BIONIC_ITEM: &'static [Self] = &[Self("BIONIC_ITEM")];
    pub(crate) const BOOK: &'static [Self] = &[Self("BOOK")];
    pub(crate) const CLOTHING: &'static [Self] = &[Self("ARMOR")];
    pub(crate) const COMESTIBLE: &'static [Self] = &[Self("COMESTIBLE")];
    pub(crate) const ENGINE: &'static [Self] = &[Self("ENGINE")];
    pub(crate) const GENERIC_ITEM: &'static [Self] = &[Self("GENERIC")];
    pub(crate) const GUN: &'static [Self] = &[Self("GUN")];
    pub(crate) const GUNMOD: &'static [Self] = &[Self("GUNMOD")];
    pub(crate) const MAGAZINE: &'static [Self] = &[Self("MAGAZINE")];
    pub(crate) const PET_ARMOR: &'static [Self] = &[Self("PET_ARMOR")];
    pub(crate) const TOOL: &'static [Self] = &[Self("TOOL")];
    pub(crate) const TOOL_CLOTHING: &'static [Self] = &[Self("TOOL_ARMOR")];
    pub(crate) const TOOLMOD: &'static [Self] = &[Self("TOOLMOD")];
    pub(crate) const WHEEL: &'static [Self] = &[Self("WHEEL")];

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

    pub(crate) const RECIPE: &'static [Self] = &[Self("recipe")];
    pub(crate) const REQUIREMENT: &'static [Self] = &[Self("requirement")];
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
        Self("BATTERY"), // not used in CDDA 0.G (yet?)
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
        Self("recipe_category"),
        Self("recipe_group"),
        Self("SPELL"),
        Self("ter_furn_transform"),
        Self("TRAIT_MIGRATION"),
        Self("trap"),
        Self("uncraft"),
        Self("vehicle_part_category"),
    ];

    pub(crate) const fn all() -> &'static [&'static [Self]] {
        &[
            Self::AMMO,
            Self::BIONIC_ITEM,
            Self::BOOK,
            Self::CHARACTER,
            Self::CLOTHING,
            Self::COMESTIBLE,
            Self::ENGINE,
            Self::FIELD,
            Self::FURNITURE,
            Self::ITEM_GROUP,
            Self::GENERIC_ITEM,
            Self::GUN,
            Self::GUNMOD,
            Self::MAGAZINE,
            Self::MIGRATION,
            Self::OVERMAP,
            Self::PET_ARMOR,
            Self::RECIPE,
            Self::REQUIREMENT,
            Self::TERRAIN,
            Self::TOOL,
            Self::TOOL_CLOTHING,
            Self::TOOL_QUALITY,
            Self::TOOLMOD,
            Self::VEHICLE_PART,
            Self::WHEEL,
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
