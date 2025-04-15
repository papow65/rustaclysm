use crate::{CommonItemInfo, Flags, RequiredLinkedLater};
use bevy_log::error;
use bevy_platform::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(from = "Option<serde_json::Value>")]
pub struct ExamineActionOption(pub Option<ExamineAction>);

impl From<Option<serde_json::Value>> for ExamineActionOption {
    fn from(source: Option<serde_json::Value>) -> Self {
        Self(source.and_then(|source| {
            (if source.is_string() {
                serde_json::from_value::<SimpleExamineAction>(source).map(ExamineAction::Simple)
            } else {
                serde_json::from_value::<ExamineAction>(source)
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
        effect_on_conditions: serde_json::Value,
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
