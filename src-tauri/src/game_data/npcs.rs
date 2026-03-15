use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct NpcPreference {
    pub name: Option<String>,
    pub desire: String,
    pub keywords: Vec<String>,
    pub pref: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct NpcInfo {
    pub key: String,
    pub name: String,
    pub desc: Option<String>,
    pub area_name: Option<String>,
    pub area_friendly_name: Option<String>,
    pub trains_skills: Vec<String>,
    pub preferences: Vec<NpcPreference>,
    pub item_gifts: Vec<String>,

    // Phase 4 typed fields
    pub pos: Option<Value>,
    pub services: Option<Vec<Value>>,

    // Full raw JSON
    pub raw_json: Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, NpcInfo>, String> {
    let raw: HashMap<String, serde_json::Value> = serde_json::from_str(json)
        .map_err(|e| format!("npcs.json: parse error at line {}, col {}: {e}", e.line(), e.column()))?;

    let mut npcs = HashMap::with_capacity(raw.len());
    for (key, value) in raw {
        // Basic stub: just store the key and name if available
        let name = value.get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or(&key)
            .to_string();

        let desc = value.get("Description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let area_name = value.get("AreaName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let area_friendly_name = value.get("AreaFriendlyName")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract skills from Services array where Type == "Training"
        let trains_skills = value.get("Services")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|service| {
                        service.get("Type")
                            .and_then(|t| t.as_str())
                            .map(|t| t == "Training")
                            .unwrap_or(false)
                    })
                    .flat_map(|service| {
                        service.get("Skills")
                            .and_then(|s| s.as_array())
                            .map(|skills| {
                                skills.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse preferences
        let preferences = value.get("Preferences")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|pref_val| {
                        Some(NpcPreference {
                            name: pref_val.get("Name")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            desire: pref_val.get("Desire")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string(),
                            keywords: pref_val.get("Keywords")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect())
                                .unwrap_or_default(),
                            pref: pref_val.get("Pref")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0) as f32,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse item gifts (favorites)
        let item_gifts = value.get("ItemGifts")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect())
            .unwrap_or_default();

        let pos = value.get("Pos").cloned();
        let services = value.get("Services")
            .and_then(|v| v.as_array().cloned());

        npcs.insert(key.clone(), NpcInfo {
            key,
            name,
            desc,
            area_name,
            area_friendly_name,
            trains_skills,
            preferences,
            item_gifts,
            pos,
            services,
            raw_json: value,
        });
    }

    Ok(npcs)
}
