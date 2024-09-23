use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
//#[serde(deny_unknown_fields)] // can't be used with '#[serde(flatten)]' (at the bottom)
pub struct CddaPlayer {
    pub str_max: i16,
    pub str_bonus: i16,
    pub str_cur: i16,
    pub dex_max: i16,
    pub dex_bonus: i16,
    pub dex_cur: i16,
    pub int_max: i16,
    pub int_bonus: i16,
    pub int_cur: i16,
    pub per_max: i16,
    pub per_bonus: i16,
    pub per_cur: i16,
    //pub active_mission: serde_json::Value,
    //pub active_missions: serde_json::Value,
    //pub activity: serde_json::Value,
    //pub activity_history: serde_json::Value,
    //pub activity_vehicle_part_index: serde_json::Value,
    //pub addictions: serde_json::Value,
    //pub archery_aim_counter: serde_json::Value,
    //pub armor_bash_bonus: serde_json::Value,
    //pub armor_bullet_bonus: serde_json::Value,
    //pub armor_cut_bonus: serde_json::Value,
    //pub assigned_invlet: serde_json::Value,
    //pub automoveroute: serde_json::Value,
    //pub avg_nat_bpm: serde_json::Value,
    //pub backlog: serde_json::Value,
    //pub base_age: serde_json::Value,
    //pub base_height: serde_json::Value,
    //pub bash_bonus: serde_json::Value,
    //pub bash_mult: serde_json::Value,
    //pub block_bonus: serde_json::Value,
    //pub blocks_left: serde_json::Value,
    //pub blood_rh_factor: serde_json::Value,
    //pub blood_type: serde_json::Value,
    pub body: serde_json::Value,
    //pub calorie_diary: serde_json::Value,
    //pub camps: serde_json::Value,
    //pub cardio_acc: serde_json::Value,
    //pub cash: serde_json::Value,
    //pub completed_missions: serde_json::Value,
    //pub consumption_history: serde_json::Value,
    //pub continuous_sleep: serde_json::Value,
    //pub controlling_vehicle: serde_json::Value,
    //pub custom_profession: serde_json::Value,
    //pub cut_bonus: serde_json::Value,
    //pub cut_mult: serde_json::Value,
    //pub daily_sleep: serde_json::Value,
    //pub daily_vitamins: serde_json::Value,
    //pub damage_over_time_map: serde_json::Value,
    //pub death_eocs: serde_json::Value,
    //pub destination_activity: serde_json::Value,
    //pub destination_point: serde_json::Value,
    //pub dodge_bonus: serde_json::Value,
    //pub dodges_left: serde_json::Value,
    //pub effects: serde_json::Value,
    //pub faction_warnings: serde_json::Value,
    //pub failed_missions: serde_json::Value,
    //pub fatigue: serde_json::Value,
    pub focus_pool: serde_json::Value,
    //pub followers: serde_json::Value,
    //pub grab_point: serde_json::Value,
    //pub grab_type: serde_json::Value,
    //pub guts: serde_json::Value,
    //pub health_tally: serde_json::Value,
    //pub healthy: serde_json::Value,
    //pub healthy_mod: serde_json::Value,
    //pub hit_bonus: serde_json::Value,
    //pub hunger: serde_json::Value,
    pub id: serde_json::Value,
    //pub in_vehicle: serde_json::Value,
    //pub inactive_eocs: serde_json::Value,
    //pub inv: serde_json::Value,
    //pub invcache: serde_json::Value,
    pub items_identified: serde_json::Value,
    //pub kill_xp: serde_json::Value,
    //pub known_monsters: serde_json::Value,
    //pub known_traps: serde_json::Value,
    //pub last_sleep_check: serde_json::Value,
    //pub last_target_pos: serde_json::Value,
    //pub last_updated: serde_json::Value,
    //pub learned_recipes: serde_json::Value,
    //pub location: serde_json::Value,
    //pub magic: serde_json::Value,
    //pub male: serde_json::Value,
    //pub martial_arts_data: serde_json::Value,
    //pub max_power_level_modifier: Option<serde_json::Value>,
    //pub melee_quiet: serde_json::Value,
    //pub moncams: Option<serde_json::Value>,
    //pub morale: serde_json::Value,
    //pub move_mode: serde_json::Value,
    //pub moves: serde_json::Value,
    //pub mutations: serde_json::Value,
    //pub my_bionics: serde_json::Value,
    pub name: String,
    //pub num_blocks_bonus: serde_json::Value,
    //pub num_dodges_bonus: serde_json::Value,
    pub omt_path: serde_json::Value,
    pub oxygen: serde_json::Value,
    pub pain: serde_json::Value,
    pub pkill: serde_json::Value,
    pub play_name: serde_json::Value,
    //pub power_level: serde_json::Value,
    //pub power_prev_turn: serde_json::Value,
    //pub preferred_aiming_mode: serde_json::Value,
    //pub profession: serde_json::Value,
    //pub proficiencies: serde_json::Value,
    //pub queued_effect_on_conditions: serde_json::Value,
    //pub radiation: serde_json::Value,
    //pub recoil: serde_json::Value,
    //pub scenario: serde_json::Value,
    //pub scent: serde_json::Value,
    //pub show_map_memory: serde_json::Value,
    pub skills: HashMap<String, Skill>,
    pub sleep_deprivation: serde_json::Value,
    //pub slow_rad: serde_json::Value,
    //pub snippets_read: serde_json::Value,
    //pub speed: serde_json::Value,
    //pub speed_bonus: serde_json::Value,
    //pub spent_upgrade_points: serde_json::Value,
    pub stamina: serde_json::Value,
    //pub stashed_outbounds_activity: serde_json::Value,
    //pub stashed_outbounds_backlog: serde_json::Value,
    //pub stim: serde_json::Value,
    //pub stomach: serde_json::Value,
    //pub stored_calories: serde_json::Value,
    //pub thirst: serde_json::Value,
    //pub throw_resist: serde_json::Value,
    pub traits: serde_json::Value,
    //pub translocators: serde_json::Value,
    //pub type_of_scent: serde_json::Value,
    //pub underwater: serde_json::Value,
    //pub values: serde_json::Value,
    //pub vitamin_levels: serde_json::Value,
    //pub worn: serde_json::Value,

    // To prevent a linking eror when there Too many fields to deserialize
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Skill {
    pub level: u8,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
