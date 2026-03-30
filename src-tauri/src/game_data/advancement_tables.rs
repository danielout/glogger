use super::parse_string_map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawAdvancementTableInfo {
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct AdvancementTableInfo {
    pub raw: serde_json::Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, AdvancementTableInfo>, String> {
    let raw: HashMap<String, serde_json::Value> = parse_string_map(json, "advancementtables.json")?;
    Ok(raw
        .into_iter()
        .map(|(k, v)| (k, AdvancementTableInfo { raw: v }))
        .collect())
}
