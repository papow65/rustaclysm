use bevy::utils::HashMap;
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
//#[serde(deny_unknown_fields)] // can't be used with '#[serde(flatten)]' (at the bottom)
pub(crate) struct CddaPlayer {
    pub(crate) str_max: i16,
    pub(crate) str_bonus: i16,
    pub(crate) str_cur: i16,
    pub(crate) dex_max: i16,
    pub(crate) dex_bonus: i16,
    pub(crate) dex_cur: i16,
    pub(crate) int_max: i16,
    pub(crate) int_bonus: i16,
    pub(crate) int_cur: i16,
    pub(crate) per_max: i16,
    pub(crate) per_bonus: i16,
    pub(crate) per_cur: i16,
    /*pub(crate) active_mission: serde_json::Value,
    pub(crate) active_missions: serde_json::Value,
    pub(crate) activity: serde_json::Value,
    pub(crate) activity_history: serde_json::Value,
    pub(crate) activity_vehicle_part_index: serde_json::Value,
    pub(crate) addictions: serde_json::Value,
    pub(crate) archery_aim_counter: serde_json::Value,
    pub(crate) armor_bash_bonus: serde_json::Value,
    pub(crate) armor_bullet_bonus: serde_json::Value,
    pub(crate) armor_cut_bonus: serde_json::Value,
    pub(crate) assigned_invlet: serde_json::Value,
    pub(crate) automoveroute: serde_json::Value,
    pub(crate) avg_nat_bpm: serde_json::Value,
    pub(crate) backlog: serde_json::Value,
    pub(crate) base_age: serde_json::Value,
    pub(crate) base_height: serde_json::Value,
    pub(crate) bash_bonus: serde_json::Value,
    pub(crate) bash_mult: serde_json::Value,
    pub(crate) block_bonus: serde_json::Value,
    pub(crate) blocks_left: serde_json::Value,
    pub(crate) blood_rh_factor: serde_json::Value,
    pub(crate) blood_type: serde_json::Value,*/
    pub(crate) body: serde_json::Value,
    /*pub(crate) calorie_diary: serde_json::Value,
    pub(crate) camps: serde_json::Value,
    pub(crate) cardio_acc: serde_json::Value,
    pub(crate) cash: serde_json::Value,
    pub(crate) completed_missions: serde_json::Value,
    pub(crate) consumption_history: serde_json::Value,
    pub(crate) continuous_sleep: serde_json::Value,
    pub(crate) controlling_vehicle: serde_json::Value,
    pub(crate) custom_profession: serde_json::Value,
    pub(crate) cut_bonus: serde_json::Value,
    pub(crate) cut_mult: serde_json::Value,
    pub(crate) daily_sleep: serde_json::Value,
    pub(crate) daily_vitamins: serde_json::Value,
    pub(crate) damage_over_time_map: serde_json::Value,
    pub(crate) death_eocs: serde_json::Value,
    pub(crate) destination_activity: serde_json::Value,
    pub(crate) destination_point: serde_json::Value,*/
    /*pub(crate) dodge_bonus: serde_json::Value,
    pub(crate) dodges_left: serde_json::Value,
    pub(crate) effects: serde_json::Value,
    pub(crate) faction_warnings: serde_json::Value,
    pub(crate) failed_missions: serde_json::Value,*/
    pub(crate) fatigue: serde_json::Value,
    pub(crate) focus_pool: serde_json::Value,
    /*pub(crate) followers: serde_json::Value,
    pub(crate) grab_point: serde_json::Value,
    pub(crate) grab_type: serde_json::Value,
    pub(crate) guts: serde_json::Value,
    pub(crate) health_tally: serde_json::Value,
    pub(crate) healthy: serde_json::Value,
    pub(crate) healthy_mod: serde_json::Value,
    pub(crate) hit_bonus: serde_json::Value,*/
    pub(crate) hunger: serde_json::Value,
    pub(crate) id: serde_json::Value,
    /*pub(crate) in_vehicle: serde_json::Value,
    pub(crate) inactive_eocs: serde_json::Value,*/
    pub(crate) inv: serde_json::Value,
    //pub(crate) invcache: serde_json::Value,
    pub(crate) items_identified: serde_json::Value,
    /*pub(crate) kill_xp: serde_json::Value,
    pub(crate) known_monsters: serde_json::Value,
    pub(crate) known_traps: serde_json::Value,
    pub(crate) last_sleep_check: serde_json::Value,
    pub(crate) last_target_pos: serde_json::Value,
    pub(crate) last_updated: serde_json::Value,*/
    pub(crate) learned_recipes: serde_json::Value,
    pub(crate) location: serde_json::Value,
    pub(crate) magic: serde_json::Value,
    pub(crate) male: serde_json::Value,
    pub(crate) martial_arts_data: serde_json::Value,
    pub(crate) max_power_level_modifier: serde_json::Value,
    pub(crate) melee_quiet: serde_json::Value,
    pub(crate) moncams: serde_json::Value,
    pub(crate) morale: serde_json::Value,
    pub(crate) move_mode: serde_json::Value,
    pub(crate) moves: serde_json::Value,
    pub(crate) mutations: serde_json::Value,
    pub(crate) my_bionics: serde_json::Value,
    pub(crate) name: String,
    pub(crate) num_blocks_bonus: serde_json::Value,
    pub(crate) num_dodges_bonus: serde_json::Value,
    pub(crate) omt_path: serde_json::Value,
    pub(crate) oxygen: serde_json::Value,
    pub(crate) pain: serde_json::Value,
    pub(crate) pkill: serde_json::Value,
    pub(crate) play_name: serde_json::Value,
    /*pub(crate) power_level: serde_json::Value,
    pub(crate) power_prev_turn: serde_json::Value,
    pub(crate) preferred_aiming_mode: serde_json::Value,
    pub(crate) profession: serde_json::Value,
    pub(crate) proficiencies: serde_json::Value,
    pub(crate) queued_effect_on_conditions: serde_json::Value,
    pub(crate) radiation: serde_json::Value,
    pub(crate) recoil: serde_json::Value,
    pub(crate) scenario: serde_json::Value,
    pub(crate) scent: serde_json::Value,
    pub(crate) show_map_memory: serde_json::Value,*/
    pub(crate) skills: serde_json::Value,
    pub(crate) sleep_deprivation: serde_json::Value,
    /*pub(crate) slow_rad: serde_json::Value,
    pub(crate) snippets_read: serde_json::Value,*/
    pub(crate) speed: serde_json::Value,
    pub(crate) speed_bonus: serde_json::Value,
    //pub(crate) spent_upgrade_points: serde_json::Value,
    pub(crate) stamina: serde_json::Value,
    /*pub(crate) stashed_outbounds_activity: serde_json::Value,
    pub(crate) stashed_outbounds_backlog: serde_json::Value,
    pub(crate) stim: serde_json::Value,
    pub(crate) stomach: serde_json::Value,*/
    pub(crate) stored_calories: serde_json::Value,
    pub(crate) thirst: serde_json::Value,
    //pub(crate) throw_resist: serde_json::Value,
    pub(crate) traits: serde_json::Value,
    /*pub(crate) translocators: serde_json::Value,
    pub(crate) type_of_scent: serde_json::Value,
    pub(crate) underwater: serde_json::Value,
    pub(crate) values: serde_json::Value,
    pub(crate) vitamin_levels: serde_json::Value,*/
    pub(crate) worn: serde_json::Value,

    // To prevent a linking eror when there Too many fields to deserialize
    #[serde(flatten)]
    pub(crate) extra: HashMap<String, serde_json::Value>,
}
