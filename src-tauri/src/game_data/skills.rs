use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::parse_string_map;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawSkill {
    #[serde(rename = "Id")]
    pub id: u32,

    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "IconId")]
    pub icon_id: Option<u32>,

    #[serde(rename = "HideWhenZero")]
    pub hide_when_zero: Option<bool>,

    #[serde(rename = "XpTable")]
    pub xp_table: Option<String>,

    #[serde(rename = "AdvancementTable")]
    pub advancement_table: Option<String>,

    #[serde(rename = "Keywords")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "Rewards")]
    pub rewards: Option<serde_json::Value>,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

/// A single skill definition.
#[derive(Debug, Serialize, Clone)]
pub struct SkillInfo {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub icon_id: Option<u32>,
    pub xp_table: Option<String>,
    pub keywords: Vec<String>,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<u32, SkillInfo>, String> {
    // Skills.json uses skill names as keys (e.g., "Alchemy", "Cooking")
    // Each skill has an "Id" field inside
    let raw_skills: HashMap<String, RawSkill> = parse_string_map(json, "skills.json")?;
    eprintln!("skills.json: Parsed {} raw skills", raw_skills.len());

    let mut skills = HashMap::with_capacity(raw_skills.len());
    for (skill_name, raw) in raw_skills {
        let id = raw.id;
        skills.insert(id, SkillInfo {
            id,
            // Prefer the Name field, fall back to the key name
            name: raw.name.unwrap_or(skill_name),
            description: raw.description,
            icon_id: raw.icon_id,
            xp_table: raw.xp_table,
            keywords: raw.keywords.unwrap_or_default(),
        });
    }

    eprintln!("skills.json: Created {} SkillInfo entries", skills.len());
    Ok(skills)
}
