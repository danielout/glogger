use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

/// A single item definition, suitable for serialising to the frontend.
/// Contains typed fields for commonly-used data, plus the full raw JSON
/// so no CDN data is ever lost.
#[derive(Debug, Serialize, Clone)]
pub struct ItemInfo {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub icon_id: Option<u32>,
    pub value: Option<f32>,
    pub max_stack_size: Option<f32>,
    pub keywords: Vec<String>,
    pub effect_descs: Vec<String>,

    // ── New typed fields (Phase 1) ──────────────────────────────────────
    pub internal_name: Option<String>,
    pub food_desc: Option<String>,
    pub equip_slot: Option<String>,
    pub num_uses: Option<u32>,
    pub skill_reqs: Option<Value>,
    pub behaviors: Option<Vec<Value>>,
    pub bestow_recipes: Option<Vec<Value>>,
    pub bestow_ability: Option<String>,
    pub bestow_quest: Option<String>,
    pub bestow_title: Option<u32>,
    pub craft_points: Option<u32>,
    pub crafting_target_level: Option<u32>,
    pub tsys_profile: Option<String>,

    // ── Full raw JSON (source of truth) ─────────────────────────────────
    pub raw_json: Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, ItemInfo>, String> {
    let raw: HashMap<String, Value> = serde_json::from_str(json).map_err(|e| {
        format!(
            "items.json: parse error at line {}, col {}: {e}",
            e.line(),
            e.column()
        )
    })?;

    let mut items = HashMap::with_capacity(raw.len());
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

        let item = ItemInfo {
            id,
            name: str_field(&value, "Name").unwrap_or_else(|| format!("Unknown Item {id}")),
            description: str_field(&value, "Description"),
            icon_id: u32_field(&value, "IconId"),
            value: f32_field(&value, "Value"),
            max_stack_size: f32_field(&value, "MaxStackSize"),
            keywords: str_array_field(&value, "Keywords"),
            effect_descs: str_array_field(&value, "EffectDescs"),

            // Phase 1 typed fields
            internal_name: str_field(&value, "InternalName"),
            food_desc: str_field(&value, "FoodDesc"),
            equip_slot: str_field(&value, "EquipSlot"),
            num_uses: u32_field(&value, "NumUses"),
            skill_reqs: value.get("SkillReqs").cloned(),
            behaviors: value.get("Behaviors").and_then(|v| v.as_array().cloned()),
            bestow_recipes: value
                .get("BestowRecipes")
                .and_then(|v| v.as_array().cloned()),
            bestow_ability: str_field(&value, "BestowAbility"),
            bestow_quest: str_field(&value, "BestowQuest"),
            bestow_title: u32_field(&value, "BestowTitle"),
            craft_points: u32_field(&value, "CraftPoints"),
            crafting_target_level: u32_field(&value, "CraftingTargetLevel"),
            tsys_profile: str_field(&value, "TSysProfile"),

            raw_json: value,
        };

        items.insert(id, item);
    }

    if skipped > 0 {
        eprintln!("items.json: Warning: skipped {skipped} entries with invalid keys");
    }

    Ok(items)
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
