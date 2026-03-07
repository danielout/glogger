use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::parse_id_map;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawItem {
    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "IconId")]
    pub icon_id: Option<u32>,

    #[serde(rename = "Value")]
    pub value: Option<f32>,

    #[serde(rename = "MaxStackSize")]
    pub max_stack_size: Option<f32>,

    #[serde(rename = "Keywords")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "EffectDescs")]
    pub effect_descs: Option<Vec<String>>,

    #[serde(rename = "DroppedAppearance")]
    pub dropped_appearance: Option<String>,

    #[serde(rename = "FoodDesc")]
    pub food_desc: Option<String>,

    #[serde(rename = "SkillReqs")]
    pub skill_reqs: Option<serde_json::Value>,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

/// A single item definition, suitable for serialising to the frontend.
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
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, ItemInfo>, String> {
    let raw_items: HashMap<u32, RawItem> = parse_id_map(json, "items.json")?;

    let mut items = HashMap::with_capacity(raw_items.len());
    for (id, raw) in raw_items {
        items.insert(id, ItemInfo {
            id,
            name: raw.name.unwrap_or_else(|| format!("Unknown Item {id}")),
            description: raw.description,
            icon_id: raw.icon_id,
            value: raw.value,
            max_stack_size: raw.max_stack_size,
            keywords: raw.keywords.unwrap_or_default(),
            effect_descs: raw.effect_descs.unwrap_or_default(),
        });
    }

    Ok(items)
}
