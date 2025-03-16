use serde::Deserialize;

#[derive(Clone, Hash, PartialEq, Eq, Debug, Deserialize)]
pub(crate) struct TypeId(&'static str);

impl TypeId {
    // Use this shell command to list all json types used:
    // find assets/data/json/ -type f | xargs -I {} jq '.[].type' {} 2>/dev/null | sort -u

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

    pub(crate) const OVERMAP_TERRAIN: &'static [Self] = &[Self("overmap_terrain")];

    pub(crate) const RECIPE: &'static [Self] = &[Self("recipe")];
    pub(crate) const REQUIREMENT: &'static [Self] = &[Self("requirement")];
    pub(crate) const TERRAIN: &'static [Self] = &[Self("terrain")];
    pub(crate) const TOOL_QUALITY: &'static [Self] = &[Self("tool_quality")];
    pub(crate) const VEHICLE_PART: &'static [Self] = &[Self("vehicle_part")];

    pub(crate) const ITEM_MIGRATION: &'static [Self] = &[Self("MIGRATION")];
    pub(crate) const VEHICLE_PART_MIGRATION: &'static [Self] = &[Self("vehicle_part_migration")];

    // TODO use these types
    pub(crate) const UNUSED: &'static [Self] = &[
        Self("achievement"),
        Self("activity_type"),
        Self("addiction_type"),
        Self("ammo_effect"),
        Self("ammunition_type"),
        Self("anatomy"),
        Self("ascii_art"),
        Self("BATTERY"), // not used in CDDA 0.G (yet?)
        Self("behavior"),
        Self("bionic"),
        Self("body_graph"),
        Self("body_part"),
        Self("butchery_requirement"),
        Self("character_mod"),
        Self("charge_migration_blacklist"),
        Self("charge_removal_blacklist"),
        Self("city_building"), // typically span multiple overmap terrains
        Self("clothing_mod"),
        Self("conduct"),
        Self("connect_group"),
        Self("construction"),
        Self("construction_category"),
        Self("construction_group"),
        Self("disease_type"),
        Self("dream"),
        Self("effect_on_condition"),
        Self("effect_type"),
        Self("emit"),
        Self("enchantment"),
        Self("event_statistic"),
        Self("event_transformation"),
        Self("faction"),
        Self("fault"),
        Self("field_type"),
        Self("gate"),
        Self("harvest"),
        Self("harvest_drop_type"),
        Self("hit_range"),
        Self("item_action"),
        Self("ITEM_CATEGORY"),
        Self("json_flag"),
        Self("limb_score"),
        Self("LOOT_ZONE"),
        Self("map_extra"), // notes on the overmap
        Self("mapgen"),
        Self("martial_art"),
        Self("material"),
        Self("mission_definition"),
        Self("monster_attack"),
        Self("MONSTER_BLACKLIST"),
        Self("MONSTER_FACTION"),
        Self("monster_flag"),
        Self("monstergroup"),
        Self("mood_face"),
        Self("morale_type"),
        Self("movement_mode"),
        Self("mutation"),
        Self("mutation_category"),
        Self("mutation_type"),
        Self("nested_category"),
        Self("npc"),
        Self("npc_class"),
        Self("obsolete_terrain"),
        Self("overlay_order"),
        Self("overmap_connection"), // like roads
        Self("overmap_land_use_code"),
        Self("overmap_location"),
        Self("overmap_special"), // typically span multiple overmap terrains
        Self("overmap_special_migration"),
        Self("palette"),
        Self("practice"),
        Self("profession"),
        Self("profession_item_substitutions"),
        Self("proficiency"),
        Self("proficiency_category"),
        Self("recipe_category"),
        Self("recipe_group"),
        Self("region_settings"),
        Self("relic_procgen_data"),
        Self("rotatable_symbol"),
        Self("scenario"),
        Self("scent_type"),
        Self("score"),
        Self("shopkeeper_blacklist"),
        Self("shopkeeper_consumption_rates"),
        Self("skill"),
        Self("skill_display_type"),
        Self("snippet"),
        Self("SPECIES"),
        Self("speech"),
        Self("speed_description"),
        Self("SPELL"),
        Self("start_location"),
        Self("sub_body_part"),
        Self("talk_topic"),
        Self("technique"),
        Self("ter_furn_transform"),
        Self("trait_group"),
        Self("TRAIT_MIGRATION"),
        Self("trap"),
        Self("uncraft"),
        Self("vehicle"),
        Self("vehicle_group"),
        Self("vehicle_part_category"),
        Self("vehicle_placement"),
        Self("vehicle_spawn"),
        Self("vitamin"),
        Self("weakpoint_set"),
        Self("weapon_category"),
        Self("weather_type"),
        Self("widget"),
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
            Self::ITEM_MIGRATION,
            Self::GENERIC_ITEM,
            Self::GUN,
            Self::GUNMOD,
            Self::MAGAZINE,
            Self::OVERMAP_TERRAIN,
            Self::PET_ARMOR,
            Self::RECIPE,
            Self::REQUIREMENT,
            Self::TERRAIN,
            Self::TOOL,
            Self::TOOL_CLOTHING,
            Self::TOOL_QUALITY,
            Self::TOOLMOD,
            Self::VEHICLE_PART,
            Self::VEHICLE_PART_MIGRATION,
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
