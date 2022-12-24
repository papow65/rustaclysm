use crate::prelude::*;
use bevy::prelude::AlphaMode;

const SEPARATION_OFFSET: f32 = 0.005;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum ObjectSpecifier {
    Terrain,
    Furniture,
    Item,
    Character,
    ZoneLevel,
    Meta,
}

/*
for f in $(find assets/data/json/ -type f) ; do jq -r '.[].type' $f ; done | sort -u*

achievement
activity_type
addiction_type
AMMO
ammo_effect
ammunition_type
anatomy
ARMOR
ascii_art
BATTERY
behavior
bionic
BIONIC_ITEM
body_graph
body_part
BOOK
butchery_requirement
character_mod
charge_migration_blacklist
charge_removal_blacklist
city_building
clothing_mod
COMESTIBLE
conduct
construction
construction_category
construction_group
disease_type
dream
effect_on_condition
effect_type
emit
enchantment
ENGINE
event_statistic
event_transformation
faction
fault
field_type
furniture
gate
GENERIC
GUN
GUNMOD
harvest
harvest_drop_type
hit_range
item_action
ITEM_CATEGORY
item_group
json_flag
limb_score
LOOT_ZONE
MAGAZINE
map_extra
mapgen
martial_art
material
MIGRATION
mission_definition
MONSTER
monster_attack
MONSTER_BLACKLIST
MONSTER_FACTION
monstergroup
mood_face
morale_type
movement_mode
mutation
mutation_category
mutation_type
nested_category
npc
npc_class
obsolete_terrain
overlay_order
overmap_connection
overmap_land_use_code
overmap_location
overmap_special
overmap_special_migration
overmap_terrain
palette
PET_ARMOR
practice
profession
profession_item_substitutions
proficiency
proficiency_category
recipe
recipe_category
recipe_group
region_settings
relic_procgen_data
requirement
rotatable_symbol
scenario
scent_type
score
shopkeeper_blacklist
shopkeeper_consumption_rates
skill
skill_display_type
snippet
SPECIES
speech
speed_description
SPELL
start_location
sub_body_part
talk_topic
technique
ter_furn_transform
terrain
TOOL
TOOL_ARMOR
TOOLMOD
tool_quality
trait_group
TRAIT_MIGRATION
trap
uncraft
vehicle
vehicle_group
vehicle_part
vehicle_part_category
vehicle_part_migration
vehicle_placement
vehicle_spawn
vitamin
weakpoint_set
weapon_category
weather_type
WHEEL
widget
*/

impl ObjectSpecifier {
    pub(crate) const fn shading_applied(&self) -> bool {
        !matches!(self, Self::ZoneLevel | Self::Meta)
    }

    pub(crate) fn vertical_offset(&self, layer: &SpriteLayer) -> f32 {
        let level = match self {
            Self::ZoneLevel => -1,
            Self::Terrain => 0,
            Self::Furniture => 2,
            Self::Item => 4,
            Self::Character => 6,
            Self::Meta => 8,
        } + match &layer {
            SpriteLayer::Front => 1,
            SpriteLayer::Back => 0,
        };

        level as f32 * SEPARATION_OFFSET
    }
}

#[derive(Debug)]
pub(crate) struct ObjectDefinition<'d> {
    pub(crate) id: &'d ObjectId,
    pub(crate) specifier: ObjectSpecifier,
}

impl<'d> ObjectDefinition<'d> {
    pub(crate) fn alpha_mode(&self) -> AlphaMode {
        if self.specifier == ObjectSpecifier::Terrain && self.id.is_ground() {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        }
    }
}
