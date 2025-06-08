use crate::{CommonItemInfo, Flags, RequiredLinkedLater};
use bevy_log::error;
use bevy_platform::collections::HashMap;
use serde::Deserialize;
use serde_json::{Value as JsonValue, from_value as from_json_value};

#[derive(Debug, Deserialize)]
#[serde(from = "Option<JsonValue>")]
pub struct ExamineActionOption(pub Option<ExamineAction>);

impl From<Option<JsonValue>> for ExamineActionOption {
    fn from(source: Option<JsonValue>) -> Self {
        Self(source.and_then(|source| {
            (if source.is_string() {
                from_json_value::<SimpleExamineAction>(source).map(ExamineAction::Simple)
            } else {
                from_json_value::<ExamineAction>(source)
            })
            .inspect_err(|error| error!("Failed parsing ExamineAction: {error}"))
            .ok()
        }))
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ExamineAction {
    ApplianceConvert {
        /// Always "f_null"
        furn_set: String,
        item: RequiredLinkedLater<CommonItemInfo>,
    },
    Cardreader {
        allow_hacking: Option<bool>,
        consume_card: Option<bool>,
        despawn_monsters: Option<bool>,
        flags: Flags,
        query: Option<bool>,
        query_msg: String,
        radius: Option<u8>,
        redundant_msg: String,
        success_msg: String,
        terrain_changes: HashMap<String, String>,
    },
    EffectOnCondition {
        effect_on_conditions: JsonValue,
    },
    Simple(SimpleExamineAction),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimpleExamineAction {
    ArcfurnaceEmpty,
    ArcfurnaceFull,
    AggiePlant,
    Atm,
    AutoclaveEmpty,
    AutoclaveFull,
    Autodoc,
    Bars,
    BulletinBoard,
    CardreaderFp,
    CardreaderRobofac,
    Chainfence,
    ChangeAppearance,
    ControlsGate,
    Curtains,
    Cvdmachine,
    DeployedFurniture,
    Dirtmound,
    DoorPeephole,
    EggSackbw,
    EggSackcs,
    EggSackws,
    Elevator,
    FiniteWaterSource,
    Fireplace,
    FlowerCactus,
    FlowerDahlia,
    FlowerPoppy,
    FlowerMarloss,
    Fswitch,
    Fungus,
    FvatEmpty,
    FvatFull,
    Gaspump,
    GunsafeEl,
    HarvestedPlant,
    HarvestFurn,
    HarvestFurnNectar,
    HarvestPlantEx,
    HarvestTer,
    HarvestTerNectar,
    Intercom,
    Keg,
    KilnEmpty,
    KilnFull,
    Ledge,
    LockedObject,
    LockedObjectPickable,
    Nanofab,
    OpenSafe,
    PayGas,
    PedestalTemple,
    PedestalWyrm,
    Pit,
    PitCovered,
    PortableStructure,
    QuernExamine,
    ReloadFurniture,
    Rubble,
    Safe,
    ShrubMarloss,
    ShrubWildveggies,
    Sign,
    SlotMachine,
    SmokerOptions,
    TreeHickory,
    TreeMaple,
    TreeMapleTapped,
    TreeMarloss,
    Vending,
    WaterSource,
    Workbench,
    Workout,
}
