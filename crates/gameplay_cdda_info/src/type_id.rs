use serde::Deserialize;
use strum::VariantArray;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Deserialize, VariantArray)]
pub(crate) enum TypeId {
    // Use this shell command to list all json types used:
    // find assets/data/json/ -type f | xargs -I {} jq '.[].type' {} 2>/dev/null | sort -u

    // Common types
    #[serde(rename = "MONSTER")]
    Character,
    #[serde(rename = "field_type")]
    Field,
    #[serde(rename = "furniture")]
    Furniture,
    #[serde(rename = "overmap_terrain")]
    OvermapTerrain,
    #[serde(rename = "terrain")]
    Terrain,
    #[serde(rename = "vehicle_part")]
    VehiclePart,

    // Item types
    #[serde(rename = "AMMO")]
    Ammo,
    #[serde(rename = "BIONIC_ITEM")]
    BionicItem,
    #[serde(rename = "BOOK")]
    Book,

    #[serde(rename = "ARMOR")]
    Clothing,

    #[serde(rename = "COMESTIBLE")]
    Comestible,
    #[serde(rename = "ENGINE")]
    Engine,
    #[serde(rename = "GENERIC")]
    GenericItem,
    #[serde(rename = "GUN")]
    Gun,
    #[serde(rename = "GUNMOD")]
    GunMod,
    #[serde(rename = "MAGAZINE")]
    Magazine,
    #[serde(rename = "PET_ARMOR")]
    PetArmor,
    #[serde(rename = "TOOL")]
    Tool,
    #[serde(rename = "TOOL_ARMOR")]
    ToolClothing,
    #[serde(rename = "TOOLMOD")]
    ToolMod,
    #[serde(rename = "WHEEL")]
    Wheel,

    // Abstract types
    #[serde(rename = "item_action")]
    ItemAction,
    #[serde(rename = "item_group")]
    ItemGroup,
    #[serde(rename = "practice")]
    Practice,
    #[serde(rename = "recipe")]
    Recipe,
    #[serde(rename = "requirement")]
    Requirement,
    #[serde(rename = "tool_quality")]
    ToolQuality,

    // Migrations types
    #[serde(rename = "MIGRATION")]
    ItemMigration,
    #[serde(rename = "vehicle_part_migration")]
    VehiclePartMigration,

    // TODO use these types
    #[serde(rename = "achievement")]
    Achievement,
    #[serde(rename = "activity_type")]
    ActivityType,
    #[serde(rename = "addiction_type")]
    AddictionType,
    #[serde(rename = "ammo_effect")]
    AmmoEffect,
    #[serde(rename = "ammunition_type")]
    AmmunitionType,
    #[serde(rename = "anatomy")]
    Anatomy,
    #[serde(rename = "ascii_art")]
    AsciiArt,

    /// not used in CDDA 0.G (yet?)
    #[serde(rename = "BATTERY")]
    Battery,

    #[serde(rename = "behavior")]
    Behavior,
    #[serde(rename = "bionic")]
    Bionic,
    #[serde(rename = "body_graph")]
    BodyGraph,
    #[serde(rename = "body_part")]
    BodyPart,
    #[serde(rename = "butchery_requirement")]
    ButcheryRequirement,
    #[serde(rename = "character_mod")]
    CharacterMod,
    #[serde(rename = "charge_migration_blacklist")]
    ChargeMigrationBlacklist,
    #[serde(rename = "charge_removal_blacklist")]
    ChargeRemovalBlacklist,

    /// typically span multiple overmap terrains
    #[serde(rename = "city_building")]
    CityBuilding,

    #[serde(rename = "clothing_mod")]
    ClothingMod,
    #[serde(rename = "conduct")]
    Conduct,
    #[serde(rename = "connect_group")]
    ConnectGroup,
    #[serde(rename = "construction")]
    Construction,
    #[serde(rename = "construction_category")]
    ConstructionCategory,
    #[serde(rename = "construction_group")]
    ConstructionGroup,
    #[serde(rename = "disease_type")]
    DiseaseType,
    #[serde(rename = "dream")]
    Dream,
    #[serde(rename = "effect_on_condition")]
    EffectOnCondition,
    #[serde(rename = "effect_type")]
    EffectType,
    #[serde(rename = "emit")]
    Emit,
    #[serde(rename = "enchantment")]
    Enchantment,
    #[serde(rename = "event_statistic")]
    EventStatistic,
    #[serde(rename = "event_transformation")]
    EventTransformation,
    #[serde(rename = "faction")]
    Faction,
    #[serde(rename = "fault")]
    Fault,
    #[serde(rename = "gate")]
    Gate,
    #[serde(rename = "harvest")]
    Harvest,
    #[serde(rename = "harvest_drop_type")]
    HarvestDropType,
    #[serde(rename = "hit_range")]
    HitRange,
    #[serde(rename = "ITEM_CATEGORY")]
    ItemCategory,
    #[serde(rename = "json_flag")]
    JsonFlag,
    #[serde(rename = "limb_score")]
    LimbScore,
    #[serde(rename = "LOOT_ZONE")]
    LootZone,

    /// notes on the overmap
    #[serde(rename = "map_extra")]
    MapExtra,

    #[serde(rename = "mapgen")]
    Mapgen,
    #[serde(rename = "martial_art")]
    MartialArt,
    #[serde(rename = "material")]
    Material,
    #[serde(rename = "mission_definition")]
    MissionDefinition,
    #[serde(rename = "monster_attack")]
    MonsterAttack,
    #[serde(rename = "MONSTER_BLACKLIST")]
    MonsterBlacklist,
    #[serde(rename = "MONSTER_FACTION")]
    MonsterFaction,
    #[serde(rename = "monster_flag")]
    MonsterFlag,
    #[serde(rename = "monstergroup")]
    MonsterGroup,
    #[serde(rename = "mood_face")]
    MoodFace,
    #[serde(rename = "morale_type")]
    MoraleType,
    #[serde(rename = "movement_mode")]
    MovementMode,
    #[serde(rename = "mutation")]
    Mutation,
    #[serde(rename = "mutation_category")]
    MutationCategory,
    #[serde(rename = "mutation_type")]
    MutationType,
    #[serde(rename = "nested_category")]
    NestedCategory,
    #[serde(rename = "npc")]
    Npc,
    #[serde(rename = "npc_class")]
    NpcClass,
    #[serde(rename = "obsolete_terrain")]
    ObsoleteTerrain,
    #[serde(rename = "overlay_order")]
    OverlayOrder,

    /// like roads
    #[serde(rename = "overmap_connection")]
    OvermapConnection,

    #[serde(rename = "overmap_land_use_code")]
    OvermapLandUseCode,
    #[serde(rename = "overmap_location")]
    OvermapLocation,

    /// typically span multiple overmap terrains
    #[serde(rename = "overmap_special")]
    OvermapSpecial,

    #[serde(rename = "overmap_special_migration")]
    OvermapSpecialMigration,
    #[serde(rename = "palette")]
    Palette,
    #[serde(rename = "profession")]
    Profession,
    #[serde(rename = "profession_item_substitutions")]
    ProfessionItemSubstitutions,
    #[serde(rename = "proficiency")]
    Proficiency,
    #[serde(rename = "proficiency_category")]
    ProficiencyCategory,
    #[serde(rename = "recipe_category")]
    RecipeCategory,
    #[serde(rename = "recipe_group")]
    RecipeGroup,
    #[serde(rename = "region_settings")]
    RegionSettings,
    #[serde(rename = "relic_procgen_data")]
    RelicProcgenData,
    #[serde(rename = "rotatable_symbol")]
    RotatableSymbol,
    #[serde(rename = "scenario")]
    Scenario,
    #[serde(rename = "scent_type")]
    ScentType,
    #[serde(rename = "score")]
    Score,
    #[serde(rename = "shopkeeper_blacklist")]
    ShopkeeperBlacklist,
    #[serde(rename = "shopkeeper_consumption_rates")]
    ShopkeeperConsumptionRates,
    #[serde(rename = "skill")]
    Skill,
    #[serde(rename = "skill_display_type")]
    SkillDisplayType,
    #[serde(rename = "snippet")]
    Snippet,
    #[serde(rename = "SPECIES")]
    Species,
    #[serde(rename = "speech")]
    Speech,
    #[serde(rename = "speed_description")]
    SpeedDescription,
    #[serde(rename = "SPELL")]
    Spell,
    #[serde(rename = "start_location")]
    StartLocation,
    #[serde(rename = "sub_body_part")]
    SubBodyPart,
    #[serde(rename = "talk_topic")]
    TalkTopic,
    #[serde(rename = "technique")]
    Technique,
    #[serde(rename = "ter_furn_transform")]
    TerTurnTransform,
    #[serde(rename = "trait_group")]
    TraitGroup,
    #[serde(rename = "TRAIT_MIGRATION")]
    TraitMigration,
    #[serde(rename = "trap")]
    Trap,
    #[serde(rename = "uncraft")]
    Uncraft,
    #[serde(rename = "vehicle")]
    Vehicle,
    #[serde(rename = "vehicle_group")]
    VehicleGroup,
    #[serde(rename = "vehicle_part_category")]
    VehiclePartCategory,
    #[serde(rename = "vehicle_placement")]
    VehiclePlacement,
    #[serde(rename = "vehicle_spawn")]
    VehicleSpawn,
    #[serde(rename = "vitamin")]
    Vitamin,
    #[serde(rename = "weakpoint_set")]
    WeakpointSet,
    #[serde(rename = "weapon_category")]
    WeaponCategory,
    #[serde(rename = "weather_type")]
    WeatherType,
    #[serde(rename = "widget")]
    Widget,
}

impl TypeId {
    pub(crate) const fn in_use(self) -> bool {
        self as u8 <= Self::VehiclePartMigration as u8
    }
}
