use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::parse_string_map;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawItemUseInfo {
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct ItemUseInfo {
    pub raw: serde_json::Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, ItemUseInfo>, String> {
    let raw: HashMap<String, serde_json::Value> = parse_string_map(json, "itemuses.json")?;
    Ok(raw.into_iter()
        .map(|(k, v)| (k, ItemUseInfo { raw: v }))
        .collect())
}
