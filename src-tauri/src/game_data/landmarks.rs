use super::parse_string_map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Raw CDN shapes ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct RawLandmarkInfo {
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ── Parsed structs (app shape) ───────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Default)]
pub struct LandmarkInfo {
    pub raw: serde_json::Value,
}

// ── Parse function ───────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<HashMap<String, LandmarkInfo>, String> {
    let raw: HashMap<String, serde_json::Value> = parse_string_map(json, "landmarks.json")?;
    Ok(raw
        .into_iter()
        .map(|(k, v)| (k, LandmarkInfo { raw: v }))
        .collect())
}
