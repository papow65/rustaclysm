use crate::prelude::{
    AmmoType, CddaItemInfo, CddaItemName, Description, Label, Material, Materials, ObjectName,
    Price, ToHit,
};

#[derive(Debug)]
pub(crate) struct ItemInfo {
    #[allow(unused)]
    id: ObjectName,

    #[allow(unused)]
    category: String,

    #[allow(unused)]
    effects: Option<Vec<String>>,

    #[allow(unused)]
    // example: { "price": 0.7, "damage": { "damage_type": "bullet", "amount": 0.9 }, "dispersion": 1.1 }
    proportional: Option<serde_json::Value>,

    #[allow(unused)]
    // example: { "damage": { "damage_type": "bullet", "amount": -1, "armor_penetration": 2 } }
    relative: Option<serde_json::Value>,

    #[allow(unused)]
    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    shot_spread: Option<u16>,

    #[allow(unused)]
    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    damage: Option<serde_json::Value>,

    #[allow(unused)]
    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    shot_damage: Option<serde_json::Value>,

    #[allow(unused)]
    count: Option<u32>,

    #[allow(unused)]
    projectile_count: Option<u8>,

    #[allow(unused)]
    stack_size: Option<u8>,

    #[allow(unused)]
    ammo_type: Option<AmmoType>,

    #[allow(unused)]
    casing: Option<String>,

    #[allow(unused)]
    range: Option<u16>,

    #[allow(unused)]
    dispersion: Option<u16>,

    #[allow(unused)]
    recoil: Option<u16>,

    #[allow(unused)]
    loudness: Option<u16>,

    #[allow(unused)]
    drop: Option<String>,

    #[allow(unused)]
    show_stats: Option<bool>,

    // The fields below are listed in load_basic_info as item_factory.cpp:3932
    #[allow(unused)]
    weight: Option<String>,

    #[allow(unused)]
    integral_weight: Option<serde_json::Value>,

    #[allow(unused)]
    volume: Option<String>,

    #[allow(unused)]
    longest_side: Option<String>,

    #[allow(unused)]
    price: Option<Price>,

    #[allow(unused)]
    price_postapoc: Option<Price>,

    #[allow(unused)]
    stackable: Option<serde_json::Value>,
    #[allow(unused)]
    integral_volume: Option<serde_json::Value>,
    #[allow(unused)]
    integral_longest_side: Option<serde_json::Value>,

    #[allow(unused)]
    bashing: Option<u16>,

    #[allow(unused)]
    cutting: Option<u16>,

    #[allow(unused)]
    to_hit: Option<ToHit>,

    #[allow(unused)]
    variant_type: Option<serde_json::Value>,
    #[allow(unused)]
    variants: Option<serde_json::Value>,

    #[allow(unused)]
    container: Option<String>,

    #[allow(unused)]
    sealed: Option<bool>,

    #[allow(unused)]
    min_strength: Option<serde_json::Value>,
    #[allow(unused)]
    min_dexterity: Option<serde_json::Value>,
    #[allow(unused)]
    min_intelligence: Option<serde_json::Value>,
    #[allow(unused)]
    min_perception: Option<serde_json::Value>,
    #[allow(unused)]
    emits: Option<serde_json::Value>,

    #[allow(unused)]
    explode_in_fire: Option<bool>,

    #[allow(unused)]
    insulation: Option<serde_json::Value>,
    #[allow(unused)]
    solar_efficiency: Option<serde_json::Value>,
    #[allow(unused)]
    ascii_picture: Option<serde_json::Value>,
    #[allow(unused)]
    thrown_damage: Option<serde_json::Value>,
    #[allow(unused)]
    repairs_like: Option<serde_json::Value>,
    #[allow(unused)]
    weapon_category: Option<serde_json::Value>,
    #[allow(unused)]
    damage_states: Option<serde_json::Value>,
    #[allow(unused)]
    degradation_multiplier: Option<serde_json::Value>,

    #[allow(unused)]
    type_: String,

    name: ItemName,

    #[allow(unused)]
    description: Option<Description>,

    #[allow(unused)]
    symbol: Option<char>,

    #[allow(unused)]
    color: Option<String>,

    #[allow(unused)]
    material: Option<Vec<Material>>,

    #[allow(unused)]
    material_thickness: Option<f32>,

    #[allow(unused)]
    chat_topics: Option<serde_json::Value>,

    #[allow(unused)]
    phase: Option<String>,

    #[allow(unused)]
    magazines: Option<serde_json::Value>,

    #[allow(unused)]
    nanofab_template_group: Option<serde_json::Value>,

    #[allow(unused)]
    template_requirements: Option<serde_json::Value>,

    #[allow(unused)]
    min_skills: Option<serde_json::Value>,

    #[allow(unused)]
    explosion: Option<serde_json::Value>,

    #[allow(unused)]
    flags: Option<Vec<String>>,

    #[allow(unused)]
    faults: Option<serde_json::Value>,

    #[allow(unused)]
    qualities: Option<Vec<(String, i8)>>,

    #[allow(unused)]
    // example: { "effects": [ "RECYCLED" ] }
    extend: Option<serde_json::Value>,

    #[allow(unused)]
    // example: { "effects": [ "NEVER_MISFIRES" ], "flags": [ "IRREPLACEABLE_CONSUMABLE" ] }
    delete: Option<serde_json::Value>,

    #[allow(unused)]
    charged_qualities: Option<serde_json::Value>,

    #[allow(unused)]
    properties: Option<serde_json::Value>,

    #[allow(unused)]
    techniques: Option<serde_json::Value>,

    #[allow(unused)]
    max_charges: Option<u16>,

    #[allow(unused)]
    initial_charges: Option<u16>,

    #[allow(unused)]
    use_action: Option<serde_json::Value>,

    #[allow(unused)]
    countdown_interval: Option<serde_json::Value>,

    #[allow(unused)]
    countdown_destroy: Option<serde_json::Value>,

    #[allow(unused)]
    countdown_action: Option<serde_json::Value>,

    #[allow(unused)]
    drop_action: Option<serde_json::Value>,

    #[allow(unused)]
    looks_like: Option<ObjectName>,

    #[allow(unused)]
    conditional_names: Option<serde_json::Value>,

    #[allow(unused)]
    armor_data: Option<serde_json::Value>,

    #[allow(unused)]
    pet_armor_data: Option<serde_json::Value>,

    #[allow(unused)]
    book_data: Option<serde_json::Value>,

    #[allow(unused)]
    gun_data: Option<serde_json::Value>,

    #[allow(unused)]
    bionic_data: Option<serde_json::Value>,

    #[allow(unused)]
    ammo_data: Option<serde_json::Value>,

    #[allow(unused)]
    seed_data: Option<serde_json::Value>,

    #[allow(unused)]
    brewable: Option<serde_json::Value>,

    #[allow(unused)]
    relic_data: Option<serde_json::Value>,

    #[allow(unused)]
    milling: Option<serde_json::Value>,

    #[allow(unused)]
    gunmod_data: Option<serde_json::Value>,

    #[allow(unused)]
    pocket_data: Option<Vec<serde_json::Value>>,

    #[allow(unused)]
    armor: Option<Vec<serde_json::Value>>,

    #[allow(unused)]
    snippet_category: Option<serde_json::Value>,
}

impl ItemInfo {
    pub(crate) fn new(stack: &[&CddaItemInfo]) -> Self {
        assert!(!stack.is_empty());
        ItemInfo {
            id: stack[0].id.clone().unwrap(),
            category: Self::first(stack, |c| c.category.clone())
                .unwrap_or_else(|| "other".to_string()),

            effects: Self::first(stack, |c| c.effects.clone()),
            proportional: Self::first(stack, |c| c.proportional.clone()),
            relative: Self::first(stack, |c| c.relative.clone()),
            shot_spread: Self::first(stack, |c| c.shot_spread),
            damage: Self::first(stack, |c| c.damage.clone()),
            shot_damage: Self::first(stack, |c| c.shot_damage.clone()),
            count: Self::first(stack, |c| c.count),
            projectile_count: Self::first(stack, |c| c.projectile_count),
            stack_size: Self::first(stack, |c| c.stack_size),
            ammo_type: Self::first(stack, |c| c.ammo_type.clone()),
            casing: Self::first(stack, |c| c.casing.clone()),
            range: Self::first(stack, |c| c.range.map(i16::unsigned_abs)),
            dispersion: Self::first(stack, |c| c.dispersion),
            recoil: Self::first(stack, |c| c.recoil),
            loudness: Self::first(stack, |c| c.loudness),
            drop: Self::first(stack, |c| c.drop.clone()),
            show_stats: Self::first(stack, |c| c.show_stats),
            weight: Self::first(stack, |c| c.weight.clone()),
            integral_weight: Self::first(stack, |c| c.integral_weight.clone()),
            volume: Self::first(stack, |c| c.volume.clone()),
            longest_side: Self::first(stack, |c| c.longest_side.clone()),

            price: Self::first(stack, |c| c.price.clone()),
            price_postapoc: Self::first(stack, |c| c.price_postapoc.clone()),
            stackable: Self::first(stack, |c| c.stackable.clone()),
            integral_volume: Self::first(stack, |c| c.integral_volume.clone()),
            integral_longest_side: Self::first(stack, |c| c.integral_longest_side.clone()),
            bashing: Self::first(stack, |c| c.bashing),
            cutting: Self::first(stack, |c| c.cutting),
            to_hit: Self::first(stack, |c| c.to_hit.clone()),
            variant_type: Self::first(stack, |c| c.variant_type.clone()),
            variants: Self::first(stack, |c| c.variants.clone()),
            container: Self::first(stack, |c| c.container.clone()),
            sealed: Self::first(stack, |c| c.sealed),
            min_strength: Self::first(stack, |c| c.min_strength.clone()),
            min_dexterity: Self::first(stack, |c| c.min_dexterity.clone()),
            min_intelligence: Self::first(stack, |c| c.min_intelligence.clone()),
            min_perception: Self::first(stack, |c| c.min_perception.clone()),
            emits: Self::first(stack, |c| c.emits.clone()),
            explode_in_fire: Self::first(stack, |c| c.explode_in_fire),
            insulation: Self::first(stack, |c| c.insulation.clone()),
            solar_efficiency: Self::first(stack, |c| c.solar_efficiency.clone()),
            ascii_picture: Self::first(stack, |c| c.ascii_picture.clone()),
            thrown_damage: Self::first(stack, |c| c.thrown_damage.clone()),
            repairs_like: Self::first(stack, |c| c.repairs_like.clone()),
            weapon_category: Self::first(stack, |c| c.weapon_category.clone()),
            damage_states: Self::first(stack, |c| c.damage_states.clone()),
            degradation_multiplier: Self::first(stack, |c| c.degradation_multiplier.clone()),

            type_: stack[0].type_.clone(),
            name: ItemName::from(&stack[0].name),

            description: Self::first(stack, |c| c.description.clone()),
            symbol: Self::first(stack, |c| c.symbol),
            color: Self::first(stack, |c| c.color.clone()),
            material: Self::first(stack, |c| c.material.as_ref().map(Materials::to_vec)),
            material_thickness: Self::first(stack, |c| c.material_thickness),
            chat_topics: Self::first(stack, |c| c.chat_topics.clone()),
            phase: Self::first(stack, |c| c.phase.clone()),
            magazines: Self::first(stack, |c| c.magazines.clone()),
            nanofab_template_group: Self::first(stack, |c| c.nanofab_template_group.clone()),
            template_requirements: Self::first(stack, |c| c.template_requirements.clone()),
            min_skills: Self::first(stack, |c| c.min_skills.clone()),
            explosion: Self::first(stack, |c| c.explosion.clone()),
            flags: Self::first(stack, |c| c.flags.clone()),
            faults: Self::first(stack, |c| c.faults.clone()),
            qualities: Self::first(stack, |c| c.qualities.clone()),
            extend: Self::first(stack, |c| c.extend.clone()),
            delete: Self::first(stack, |c| c.delete.clone()),
            charged_qualities: Self::first(stack, |c| c.charged_qualities.clone()),
            properties: Self::first(stack, |c| c.properties.clone()),
            techniques: Self::first(stack, |c| c.techniques.clone()),
            max_charges: Self::first(stack, |c| c.max_charges),
            initial_charges: Self::first(stack, |c| c.initial_charges),
            use_action: Self::first(stack, |c| c.use_action.clone()),
            countdown_interval: Self::first(stack, |c| c.countdown_interval.clone()),
            countdown_destroy: Self::first(stack, |c| c.countdown_destroy.clone()),
            countdown_action: Self::first(stack, |c| c.countdown_action.clone()),
            drop_action: Self::first(stack, |c| c.drop_action.clone()),
            looks_like: Self::first(stack, |c| c.looks_like.clone()),
            conditional_names: Self::first(stack, |c| c.conditional_names.clone()),
            armor_data: Self::first(stack, |c| c.armor_data.clone()),
            pet_armor_data: Self::first(stack, |c| c.pet_armor_data.clone()),
            book_data: Self::first(stack, |c| c.book_data.clone()),
            gun_data: Self::first(stack, |c| c.gun_data.clone()),
            bionic_data: Self::first(stack, |c| c.bionic_data.clone()),
            ammo_data: Self::first(stack, |c| c.ammo_data.clone()),
            seed_data: Self::first(stack, |c| c.seed_data.clone()),
            brewable: Self::first(stack, |c| c.brewable.clone()),
            relic_data: Self::first(stack, |c| c.relic_data.clone()),
            milling: Self::first(stack, |c| c.milling.clone()),
            gunmod_data: Self::first(stack, |c| c.gunmod_data.clone()),
            pocket_data: Self::first(stack, |c| c.pocket_data.clone()),
            armor: Self::first(stack, |c| c.armor.clone()),
            snippet_category: Self::first(stack, |c| c.snippet_category.clone()),
        }
    }

    fn first<T, F>(stack: &[&CddaItemInfo], field_map: F) -> Option<T>
    where
        F: Fn(&&CddaItemInfo) -> Option<T>,
    {
        stack.iter().flat_map(field_map).next()
    }

    pub(crate) fn to_label(&self, amount: usize) -> Label {
        Label::new(self.name.get(amount))
    }
}

#[derive(Debug)]
pub(crate) struct ItemName {
    single: String,
    plural: String,
}

impl ItemName {
    pub(crate) fn get(&self, amount: usize) -> &'_ String {
        if amount == 1 {
            &self.single
        } else {
            &self.plural
        }
    }
}

impl From<&CddaItemName> for ItemName {
    fn from(origin: &CddaItemName) -> Self {
        match origin {
            CddaItemName::Simple(string) => ItemName {
                single: string.clone(),
                plural: string.clone() + "s",
            },
            CddaItemName::Both { str_sp, .. } => ItemName {
                single: str_sp.clone(),
                plural: str_sp.clone(),
            },
            CddaItemName::Split { str, str_pl, .. } => ItemName {
                single: str.clone(),
                plural: str_pl.clone().unwrap_or_else(|| str.clone() + "s"),
            },
        }
    }
}
