use super::parse_string_map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Raw CDN shape ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct RawStorageVault {
    #[serde(rename = "ID")]
    id: u32,
    area: Option<String>,
    grouping: Option<String>,
    npc_friendly_name: Option<String>,
    has_associated_npc: Option<bool>,
    num_slots: Option<u32>,
    levels: Option<HashMap<String, u32>>,
    slot_attribute: Option<String>,
    required_item_keywords: Option<Vec<String>>,
    requirement_description: Option<String>,
    // Dynamic chest fields
    num_slots_script_atomic: Option<String>,
    num_slots_script_atomic_max_value: Option<u32>,
    num_slots_script_atomic_min_value: Option<u32>,
    // Event-based storage (e.g., Ri-Shin)
    event_levels: Option<HashMap<String, u32>>,
    // Requirements (quest completion, interaction flags, etc.)
    requirements: Option<serde_json::Value>,
}

// ── Parsed structs (app shape) ──────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct StorageVaultInfo {
    pub id: u32,
    pub area: Option<String>,
    pub grouping: Option<String>,
    pub npc_friendly_name: Option<String>,
    pub has_associated_npc: Option<bool>,
    pub num_slots: Option<u32>,
    pub levels: Option<HashMap<String, u32>>,
    pub slot_attribute: Option<String>,
    pub required_item_keywords: Option<Vec<String>>,
    pub requirement_description: Option<String>,
    pub num_slots_script_atomic_max: Option<u32>,
    pub event_levels: Option<HashMap<String, u32>>,
    pub requirements: Option<serde_json::Value>,
}

// ── Parse function ──────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, StorageVaultInfo>, String> {
    let raw: HashMap<String, RawStorageVault> = parse_string_map(json, "storagevaults.json")?;
    Ok(raw
        .into_iter()
        .map(|(k, v)| {
            let info = StorageVaultInfo {
                id: v.id,
                area: v.area,
                grouping: v.grouping,
                npc_friendly_name: v.npc_friendly_name,
                has_associated_npc: v.has_associated_npc,
                num_slots: v.num_slots,
                levels: v.levels,
                slot_attribute: v.slot_attribute,
                required_item_keywords: v.required_item_keywords,
                requirement_description: v.requirement_description,
                num_slots_script_atomic_max: v.num_slots_script_atomic_max_value,
                event_levels: v.event_levels,
                requirements: v.requirements,
            };
            (k, info)
        })
        .collect())
}
