use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

/// Typed PvE or PvP combat stats for an ability.
#[derive(Debug, Serialize, Clone)]
pub struct CombatStats {
    pub damage: Option<f32>,
    pub power_cost: Option<f32>,
    pub range: Option<f32>,
    pub rage_cost: Option<f32>,
    pub accuracy: Option<f32>,
    pub attributes_that_delta_damage: Vec<String>,
    pub attributes_that_mod_base_damage: Vec<String>,
    pub attributes_that_mod_damage: Vec<String>,
    pub attributes_that_mod_crit_damage: Vec<String>,
    pub attributes_that_delta_power_cost: Vec<String>,
    pub attributes_that_mod_power_cost: Vec<String>,
    pub attributes_that_delta_rage: Vec<String>,
    pub attributes_that_mod_rage: Vec<String>,
    pub attributes_that_delta_taunt: Vec<String>,
    pub attributes_that_mod_taunt: Vec<String>,
    /// Any fields not explicitly typed above.
    pub extra: Value,
}

/// A single ability definition.
#[derive(Debug, Serialize, Clone)]
pub struct AbilityInfo {
    pub id: u32,
    pub name: String,
    pub internal_name: Option<String>,
    pub description: Option<String>,
    pub icon_id: Option<u32>,
    pub skill: Option<String>,
    pub level: Option<f32>,
    pub keywords: Vec<String>,

    pub damage_type: Option<String>,
    pub reset_time: Option<f32>,
    pub target: Option<String>,
    pub prerequisite: Option<String>,
    pub upgrade_of: Option<String>,
    pub is_harmless: Option<bool>,
    pub animation: Option<String>,
    pub special_info: Option<String>,
    pub works_underwater: Option<bool>,
    pub works_while_falling: Option<bool>,
    pub pve: Option<CombatStats>,
    pub pvp: Option<CombatStats>,
    pub mana_cost: Option<u32>,
    pub power_cost: Option<u32>,
    pub armor_cost: Option<u32>,
    pub health_cost: Option<u32>,
    pub range: Option<f32>,

    // Full raw JSON
    pub raw_json: Value,
}

/// A group of ability tiers that represent the same base ability at different power levels.
#[derive(Debug, Serialize, Clone)]
pub struct AbilityFamily {
    /// InternalName of the base (tier 1) ability, used as the family key.
    pub base_internal_name: String,
    /// Display name of the base ability (without tier number).
    pub base_name: String,
    pub icon_id: Option<u32>,
    pub skill: Option<String>,
    pub damage_type: Option<String>,
    /// Whether this is a monster-only ability (has Lint_MonsterAbility keyword).
    pub is_monster_ability: bool,
    /// Ordered list of tier ability IDs (ascending by level).
    pub tier_ids: Vec<u32>,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, AbilityInfo>, String> {
    let raw: HashMap<String, Value> = serde_json::from_str(json).map_err(|e| {
        format!(
            "abilities.json: parse error at line {}, col {}: {e}",
            e.line(),
            e.column()
        )
    })?;

    let mut abilities = HashMap::with_capacity(raw.len());
    let mut skipped = 0;

    for (key, value) in raw {
        let id_str = match key.split('_').last() {
            Some(s) => s.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };
        let id: u32 = match id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        abilities.insert(
            id,
            AbilityInfo {
                id,
                name: str_field(&value, "Name").unwrap_or_else(|| format!("Unknown Ability {id}")),
                internal_name: str_field(&value, "InternalName"),
                description: str_field(&value, "Description"),
                icon_id: u32_field(&value, "IconID"),
                skill: str_field(&value, "Skill"),
                level: f32_field(&value, "Level"),
                keywords: str_array_field(&value, "Keywords"),

                damage_type: str_field(&value, "DamageType"),
                reset_time: f32_field(&value, "ResetTime"),
                target: str_field(&value, "Target"),
                prerequisite: str_field(&value, "Prerequisite"),
                upgrade_of: str_field(&value, "UpgradeOf"),
                is_harmless: bool_field(&value, "IsHarmless"),
                animation: str_field(&value, "Animation"),
                special_info: str_field(&value, "SpecialInfo"),
                works_underwater: bool_field(&value, "WorksUnderwater"),
                works_while_falling: bool_field(&value, "WorksWhileFalling"),
                pve: value.get("PvE").map(parse_combat_stats),
                pvp: value.get("PvP").map(parse_combat_stats),
                mana_cost: u32_field(&value, "ManaCost"),
                power_cost: u32_field(&value, "PowerCost"),
                armor_cost: u32_field(&value, "ArmorCost"),
                health_cost: u32_field(&value, "HealthCost"),
                range: f32_field(&value, "Range"),

                raw_json: value,
            },
        );
    }

    if skipped > 0 {
        eprintln!("abilities.json: Warning: skipped {skipped} entries with invalid keys");
    }

    Ok(abilities)
}

// ── Combat stats parsing ────────────────────────────────────────────────────

/// Known keys that are extracted into typed CombatStats fields.
const COMBAT_STATS_KNOWN_KEYS: &[&str] = &[
    "Damage",
    "PowerCost",
    "Range",
    "RageCost",
    "Accuracy",
    "AttributesThatDeltaDamage",
    "AttributesThatModBaseDamage",
    "AttributesThatModDamage",
    "AttributesThatModCritDamage",
    "AttributesThatDeltaPowerCost",
    "AttributesThatModPowerCost",
    "AttributesThatDeltaRage",
    "AttributesThatModRage",
    "AttributesThatDeltaTaunt",
    "AttributesThatModTaunt",
];

fn parse_combat_stats(value: &Value) -> CombatStats {
    // Build `extra` object from fields we don't explicitly type
    let extra = if let Some(obj) = value.as_object() {
        let filtered: serde_json::Map<String, Value> = obj
            .iter()
            .filter(|(k, _)| !COMBAT_STATS_KNOWN_KEYS.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Value::Object(filtered)
    } else {
        Value::Object(serde_json::Map::new())
    };

    CombatStats {
        damage: f32_field(value, "Damage"),
        power_cost: f32_field(value, "PowerCost"),
        range: f32_field(value, "Range"),
        rage_cost: f32_field(value, "RageCost"),
        accuracy: f32_field(value, "Accuracy"),
        attributes_that_delta_damage: str_array_field(value, "AttributesThatDeltaDamage"),
        attributes_that_mod_base_damage: str_array_field(value, "AttributesThatModBaseDamage"),
        attributes_that_mod_damage: str_array_field(value, "AttributesThatModDamage"),
        attributes_that_mod_crit_damage: str_array_field(value, "AttributesThatModCritDamage"),
        attributes_that_delta_power_cost: str_array_field(value, "AttributesThatDeltaPowerCost"),
        attributes_that_mod_power_cost: str_array_field(value, "AttributesThatModPowerCost"),
        attributes_that_delta_rage: str_array_field(value, "AttributesThatDeltaRage"),
        attributes_that_mod_rage: str_array_field(value, "AttributesThatModRage"),
        attributes_that_delta_taunt: str_array_field(value, "AttributesThatDeltaTaunt"),
        attributes_that_mod_taunt: str_array_field(value, "AttributesThatModTaunt"),
        extra,
    }
}

// ── Field extraction helpers ─────────────────────────────────────────────────

fn str_field(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(|s| s.to_string())
}

fn u32_field(value: &Value, key: &str) -> Option<u32> {
    value.get(key)?.as_u64().map(|n| n as u32)
}

fn f32_field(value: &Value, key: &str) -> Option<f32> {
    value.get(key).and_then(|v| v.as_f64()).map(|n| n as f32)
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key)?.as_bool()
}

fn str_array_field(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}
