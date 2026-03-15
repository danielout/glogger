use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

/// A single ability definition.
#[derive(Debug, Serialize, Clone)]
pub struct AbilityInfo {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub icon_id: Option<u32>,
    pub skill: Option<String>,
    pub level: Option<f32>,
    pub keywords: Vec<String>,

    // Phase 3 typed fields
    pub damage_type: Option<String>,
    pub reset_time: Option<f32>,
    pub target: Option<String>,
    pub prerequisite: Option<String>,
    pub is_harmless: Option<bool>,
    pub animation: Option<String>,
    pub special_info: Option<String>,
    pub works_underwater: Option<bool>,
    pub works_while_falling: Option<bool>,
    pub pve: Option<Value>,
    pub pvp: Option<Value>,
    pub mana_cost: Option<u32>,
    pub power_cost: Option<u32>,
    pub armor_cost: Option<u32>,
    pub health_cost: Option<u32>,
    pub range: Option<f32>,

    // Full raw JSON
    pub raw_json: Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, AbilityInfo>, String> {
    let raw: HashMap<String, Value> = serde_json::from_str(json).map_err(|e| {
        format!("abilities.json: parse error at line {}, col {}: {e}", e.line(), e.column())
    })?;

    let mut abilities = HashMap::with_capacity(raw.len());
    let mut skipped = 0;

    for (key, value) in raw {
        let id_str = match key.split('_').last() {
            Some(s) => s.to_string(),
            None => { skipped += 1; continue; }
        };
        let id: u32 = match id_str.parse() {
            Ok(id) => id,
            Err(_) => { skipped += 1; continue; }
        };

        abilities.insert(id, AbilityInfo {
            id,
            name: str_field(&value, "Name")
                .unwrap_or_else(|| format!("Unknown Ability {id}")),
            description: str_field(&value, "Description"),
            icon_id: u32_field(&value, "IconID"),
            skill: str_field(&value, "Skill"),
            level: f32_field(&value, "Level"),
            keywords: str_array_field(&value, "Keywords"),

            // Phase 3 typed fields
            damage_type: str_field(&value, "DamageType"),
            reset_time: f32_field(&value, "ResetTime"),
            target: str_field(&value, "Target"),
            prerequisite: str_field(&value, "Prerequisite"),
            is_harmless: bool_field(&value, "IsHarmless"),
            animation: str_field(&value, "Animation"),
            special_info: str_field(&value, "SpecialInfo"),
            works_underwater: bool_field(&value, "WorksUnderwater"),
            works_while_falling: bool_field(&value, "WorksWhileFalling"),
            pve: value.get("PvE").cloned(),
            pvp: value.get("PvP").cloned(),
            mana_cost: u32_field(&value, "ManaCost"),
            power_cost: u32_field(&value, "PowerCost"),
            armor_cost: u32_field(&value, "ArmorCost"),
            health_cost: u32_field(&value, "HealthCost"),
            range: f32_field(&value, "Range"),

            raw_json: value,
        });
    }

    if skipped > 0 {
        eprintln!("abilities.json: Warning: skipped {skipped} entries with invalid keys");
    }

    Ok(abilities)
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
    value.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}
