use crate::{Ignored, InfoId, ItemAction, RequiredLinkedLater};
use serde::{Deserialize, de::Error as _};
use serde_json::{
    Error as JsonError, Map as JsonMap, Value as JsonValue, from_value as from_json_value,
};
use std::sync::Arc;
use units::{Duration, Mass, Volume};

#[derive(Debug, Deserialize)]
struct Empty {}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum UseAction {
    Simple(RequiredLinkedLater<ItemAction>),
    Weighted(RequiredLinkedLater<ItemAction>, u8),
    Typed(Box<TypedUseAction>),
}

impl UseAction {
    pub const fn id(&self) -> &RequiredLinkedLater<ItemAction> {
        match self {
            Self::Simple(id) | Self::Weighted(id, _) => id,
            Self::Typed(box_) => {
                let TypedUseAction { type_: id, .. } = &**box_;
                id
            }
        }
    }

    pub const fn level(&self) -> Option<u8> {
        if let Self::Weighted(_, level) = self {
            Some(*level)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "JsonMap<String, JsonValue>")]
pub struct TypedUseAction {
    #[serde(rename(deserialize = "type"))]
    pub type_: RequiredLinkedLater<ItemAction>,

    pub details: DetailedUseAction,
}

impl TryFrom<JsonMap<String, JsonValue>> for TypedUseAction {
    type Error = JsonError;

    fn try_from(mut map: JsonMap<String, JsonValue>) -> Result<Self, Self::Error> {
        use DetailedUseAction::{
            Ammobelt, AttachMolle, CastSpell, ChangeScent, ConsumeDrug, DelayedTransform,
            DeployFurn, DeployTent, DetachMolle, EffectOnConditions, Explosion, Firestarter, Heal,
            Holster, Inscribe, ManualNoise, Music, PlaceMonster, PlaceTrap, RepairItem, RevealMap,
            StrongAntibiotic, Transform, Unpack, WeighSelf,
        };

        let mut type_value = map
            .remove("type")
            .ok_or_else(|| Self::Error::custom("use action missing type field"))?;
        let mut type_ = type_value
            .as_str()
            .ok_or_else(|| Self::Error::custom("use action type should be a text field"))?;

        if type_ == "repair_item" {
            // We need the sub)type
            type_value = map.remove("item_action_type").ok_or_else(|| {
                Self::Error::custom("repair action missing item_action_type field")
            })?;
            type_ = type_value.as_str().ok_or_else(|| {
                Self::Error::custom("repair action item_action_type should be a text field")
            })?;
        }

        let json_object = JsonValue::Object(map);

        Ok(Self {
            type_: RequiredLinkedLater::new(InfoId::new(type_)),
            details: match type_ {
                "ammobelt" => Ammobelt(from_json_value(json_object)?),
                "attach_molle" => AttachMolle(from_json_value(json_object)?),
                "cast_spell" => CastSpell(from_json_value(json_object)?),
                "change_scent" => ChangeScent(from_json_value(json_object)?),
                "consume_drug" => ConsumeDrug(from_json_value(json_object)?),
                "delayed_transform" => DelayedTransform(from_json_value(json_object)?),
                "deploy_furn" => DeployFurn(from_json_value(json_object)?),
                "deploy_tent" => DeployTent(from_json_value(json_object)?),
                "detach_molle" => {
                    from_json_value::<Empty>(json_object)?;
                    DetachMolle
                }
                "effect_on_conditions" => EffectOnConditions(from_json_value(json_object)?),
                "explosion" => Explosion(from_json_value(json_object)?),
                "firestarter" => Firestarter(from_json_value(json_object)?),
                "heal" => Heal(from_json_value(json_object)?),
                "holster" => Holster(from_json_value(json_object)?),
                "inscribe" => Inscribe(from_json_value(json_object)?),
                "manualnoise" => ManualNoise(from_json_value(json_object)?),
                "musical_instrument" => Music(from_json_value(json_object)?),
                "place_monster" => PlaceMonster(from_json_value(json_object)?),
                "place_trap" => PlaceTrap(from_json_value(json_object)?),
                "repair_fabric" | "repair_metal" => RepairItem(from_json_value(json_object)?),
                "reveal_map" => RevealMap(from_json_value(json_object)?),
                "STRONG_ANTIBIOTIC" => {
                    from_json_value::<Empty>(json_object)?;
                    StrongAntibiotic
                }
                "transform" => Transform(from_json_value(json_object)?),
                "unpack" => Unpack(from_json_value(json_object)?),
                "weigh_self" => WeighSelf(from_json_value(json_object)?),
                _ => {
                    return Err(Self::Error::custom(format!(
                        "Unknown use action type '{type_:?}'"
                    )));
                }
            },
        })
    }
}

#[derive(Debug)]
pub enum DetailedUseAction {
    Ammobelt(AmmobeltDetail),
    AttachMolle(AttachMolleDetail),
    CastSpell(CastSpellDetail),
    ChangeScent(ChangeScentDetail),
    ConsumeDrug(ConsumeDrugDetail),
    DelayedTransform(DelayedTransformDetail),
    DeployFurn(DeployFurnDetail),
    DeployTent(DeployTentDetail),
    DetachMolle,
    EffectOnConditions(EffectOnConditionsDetail),
    Explosion(ExplosionDetail),
    Firestarter(FirestarterDetail),
    Heal(HealDetail),
    Holster(HolsterDetail),
    Inscribe(InscribeDetail),
    ManualNoise(ManualNoiseDetail),
    Music(MusicDetail),
    PlaceMonster(PlaceMonsterDetail),
    PlaceTrap(PlaceTrapDetail),
    RepairItem(RepairItemDetail),
    RevealMap(RevealMapDetail),
    StrongAntibiotic,
    Transform(TransformDetail),
    Unpack(UnpackDetail),
    WeighSelf(WeighSelfDetail),
}

#[derive(Debug, Deserialize)]
pub struct AmmobeltDetail {
    pub belt: Arc<str>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct AttachMolleDetail {
    pub size: u8,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct CastSpellDetail {
    pub spell_id: Arc<str>,
    pub level: u8,
    pub no_fail: bool,

    #[serde(default)]
    pub mundane: bool,
    #[serde(default)]
    pub need_wielding: bool,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeScentDetail {
    pub charges_to_use: u8,
    pub duration: Duration,
    pub effects: Vec<JsonValue>,
    pub moves: u8,
    pub scent_typeid: Arc<str>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct ConsumeDrugDetail {
    #[serde(default)]
    pub effects: Vec<JsonValue>,
    #[serde(default)]
    pub vitamins: Vec<JsonValue>,

    pub activation_message: Option<Arc<str>>,
    pub charges_needed: Option<JsonValue>,
    pub fields_produced: Option<JsonValue>,
    pub moved: Option<u8>,
    pub stat_adjustments: Option<JsonValue>,
    pub tools_needed: Option<JsonValue>,
    pub used_up_item: Option<Arc<str>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct DelayedTransformDetail {
    pub moves: u8,
    pub msg: Arc<str>,
    pub not_ready_msg: Arc<str>,
    pub target: Arc<str>,
    pub transform_age: u32,

    pub container: Option<Arc<str>>,
    pub target_charges: Option<u8>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct DeployFurnDetail {
    pub furn_type: Arc<str>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct DeployTentDetail {
    pub broken_type: Arc<str>,
    pub door_closed: Arc<str>, // TODO link
    pub door_opened: Arc<str>, // TODO link
    pub floor: Arc<str>,       // TODO link
    pub radius: u8,
    pub wall: Arc<str>, // TODO link

    pub floor_center: Option<Arc<str>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct EffectOnConditionsDetail {
    pub description: Arc<str>,
    pub effect_on_conditions: Vec<JsonValue>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct ExplosionDetail {
    pub no_deactivate_msg: Arc<str>,

    #[serde(default)]
    pub do_flashbang: bool,
    #[serde(default)]
    pub sound_volume: u8,

    pub draw_explosion_color: Option<Arc<str>>,
    pub draw_explosion_radius: Option<u8>,
    pub emp_blast_radius: Option<u8>,
    pub explosion: Option<JsonMap<String, JsonValue>>,
    pub fields_min_intensity: Option<u8>,
    pub fields_max_intensity: Option<u8>,
    pub fields_radius: Option<u8>,
    pub fields_type: Option<Arc<str>>,
    pub scrambler_blast_radius: Option<u8>,
    pub sound_msg: Option<Arc<str>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct FirestarterDetail {
    #[serde(default)]
    pub moves: u16,
    #[serde(default)]
    pub moves_slow: u16,
    #[serde(default)]
    pub need_sunlight: bool,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct HealDetail {
    pub bandages_power: Option<u8>,
    pub bite: Option<f32>,
    pub bleed: Option<u8>,
    pub disinfectant_power: Option<u8>,

    #[serde(default)]
    pub effects: Vec<JsonValue>,

    pub move_cost: u16,
    pub used_up_item: Option<JsonMap<String, JsonValue>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct HolsterDetail {
    pub holster_msg: Option<Arc<str>>,
    pub holster_prompt: Option<Arc<str>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct InscribeDetail {
    pub gerund: Arc<str>,
    pub material_restricted: bool,
    pub on_terrain: bool,
    pub verb: Arc<str>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct ManualNoiseDetail {
    pub moves: u8,
    pub no_charges_message: Arc<str>,
    pub noise: u8,
    pub noise_id: Arc<str>,
    pub noise_message: Arc<str>,
    pub noise_variant: Arc<str>,
    pub use_message: Arc<str>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct MusicDetail {
    pub description_frequency: u8,
    pub fun: i8,
    pub fun_bonus: u8,
    pub player_descriptions: Vec<Arc<str>>,
    pub speed_penalty: u8,
    pub volume: u8,

    pub npc_descriptions: Option<Vec<Arc<str>>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

/// Used for turrets, etc.
#[derive(Debug, Deserialize)]
pub struct PlaceMonsterDetail {
    pub difficulty: u8,
    pub monster_id: Arc<str>, // TODO link monster
    pub moves: u16,

    #[serde(default)]
    pub place_randomly: bool,
    #[serde(default)]
    pub skills: Vec<Arc<str>>, // TODO link skill

    pub friendly_msg: Option<Arc<str>>,
    pub hostile_msg: Option<Arc<str>>,
    pub need_charges: Option<u8>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct PlaceTrapDetail {
    pub done_message: Arc<str>,
    pub moves: u16,
    pub practice: u8,
    pub trap: Arc<str>, // TODO link furniture

    #[serde(default)]
    pub allow_under_player: bool,
    #[serde(default)]
    pub allow_underwater: bool,
    #[serde(default)]
    pub needs_solid_neighbor: bool,

    pub bury: Option<JsonMap<String, JsonValue>>,
    pub bury_question: Option<Arc<str>>,
    pub outer_layer_trap: Option<Arc<str>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct RepairItemDetail {
    pub cost_scaling: f32,
    pub materials: Vec<Arc<str>>, // TODO link material
    pub move_cost: u16,
    pub skill: Arc<str>,

    pub tool_quality: Option<i16>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct RevealMapDetail {
    pub message: Arc<str>,
    pub radius: u16,
    pub terrain: Vec<JsonValue>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct TransformDetail {
    #[serde(default)]
    pub active: bool,

    pub menu_text: Option<Arc<str>>,
    pub moves: Option<u16>,

    /// Shown when the action is performed
    pub msg: Option<Arc<str>>,

    pub need_fire: Option<u8>,

    #[serde(default)]
    pub need_wielding: bool,

    pub target: Option<Arc<str>>,
    pub target_charges: Option<u8>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct UnpackDetail {
    pub group: Arc<str>, // TODO link item group

    #[serde(default)]
    pub items_fit: bool,

    pub filthy_volume_threshold: Option<Volume>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct WeighSelfDetail {
    pub max_weight: Mass,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[cfg(test)]
mod use_action_tests {
    use super::*;
    use serde_json::from_str as from_json_str;

    #[test]
    fn action_list_works() {
        let json = include_str!("test_data/use_actions.json");
        let result = from_json_str::<Vec<UseAction>>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
