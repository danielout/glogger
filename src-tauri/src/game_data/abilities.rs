use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::parse_id_map;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawAbility {
    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "IconID")]
    pub icon_id: Option<u32>,

    #[serde(rename = "Skill")]
    pub skill: Option<String>,

    #[serde(rename = "Level")]
    pub level: Option<f32>,

    #[serde(rename = "Keywords")]
    pub keywords: Option<Vec<String>>,

    // PvE block; we keep it as raw JSON for now since the shape is complex
    #[serde(rename = "PvE")]
    pub pve: Option<serde_json::Value>,
}

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
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, AbilityInfo>, String> {
    let raw_abilities: HashMap<u32, RawAbility> = parse_id_map(json, "abilities.json")?;

    let mut abilities = HashMap::with_capacity(raw_abilities.len());
    for (id, raw) in raw_abilities {
        abilities.insert(id, AbilityInfo {
            id,
            name: raw.name.unwrap_or_else(|| format!("Unknown Ability {id}")),
            description: raw.description,
            icon_id: raw.icon_id,
            skill: raw.skill,
            level: raw.level,
            keywords: raw.keywords.unwrap_or_default(),
        });
    }

    Ok(abilities)
}
